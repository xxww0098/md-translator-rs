use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, broadcast};

use crate::error::{MdTranslatorError, Result};
use crate::provider::TranslationBackend;
use crate::types::{TranslateRequest, TranslateResponse};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DedupKey(String);

fn make_key(backend: &dyn TranslationBackend, request: &TranslateRequest) -> DedupKey {
    let mut hasher = blake3::Hasher::new();
    hasher.update(backend.name().as_bytes());
    hasher.update(&[0]);
    hasher.update(backend.cache_fingerprint().as_bytes());
    hasher.update(&[0]);
    let req_json = serde_json::to_vec(request).expect("TranslateRequest serializes");
    hasher.update(&req_json);
    DedupKey(hasher.finalize().to_hex().to_string())
}

type DedupValue = std::result::Result<Arc<TranslateResponse>, String>;
type InFlightMap = HashMap<DedupKey, broadcast::Sender<DedupValue>>;

/// Backend wrapper that deduplicates concurrent identical translation requests.
///
/// When multiple tasks submit the same request concurrently, only one actual
/// provider call is made. All waiters receive a clone of the result.
#[derive(Clone)]
pub struct DeduplicatingBackend {
    inner: Arc<dyn TranslationBackend>,
    in_flight: Arc<Mutex<InFlightMap>>,
}

impl DeduplicatingBackend {
    pub fn new(inner: Arc<dyn TranslationBackend>) -> Self {
        Self {
            inner,
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TranslationBackend for DeduplicatingBackend {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn cache_fingerprint(&self) -> String {
        self.inner.cache_fingerprint()
    }

    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
        let key = make_key(&*self.inner, &request);

        loop {
            let mut in_flight = self.in_flight.lock().await;

            if let Some(sender) = in_flight.get(&key) {
                let mut receiver = sender.subscribe();
                drop(in_flight);
                match receiver.recv().await {
                    Ok(shared) => {
                        return match shared {
                            Ok(arc) => Ok((*arc).clone()),
                            Err(msg) => Err(MdTranslatorError::Provider(msg)),
                        };
                    }
                    Err(_) => {
                        let mut in_flight = self.in_flight.lock().await;
                        in_flight.remove(&key);
                        drop(in_flight);
                        continue;
                    }
                }
            }

            let (sender, _) = broadcast::channel(1);
            in_flight.insert(key.clone(), sender.clone());
            drop(in_flight);

            let result = self.inner.translate_batch(request).await;

            let shared = match &result {
                Ok(response) => Ok(Arc::new(response.clone())),
                Err(err) => Err(err.to_string()),
            };

            let mut in_flight = self.in_flight.lock().await;
            let _ = sender.send(shared);
            in_flight.remove(&key);
            drop(in_flight);

            return result;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::types::{BatchItem, TranslateResponseItem};

    struct CountingBackend {
        provider_name: &'static str,
        fingerprint: String,
        call_count: AtomicUsize,
    }

    #[async_trait]
    impl TranslationBackend for CountingBackend {
        fn name(&self) -> &'static str {
            self.provider_name
        }

        fn cache_fingerprint(&self) -> String {
            self.fingerprint.clone()
        }

        async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            // Small delay to ensure concurrent requests overlap.
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let items = request
                .items
                .into_iter()
                .map(|item| TranslateResponseItem {
                    id: item.id,
                    text: format!("translated: {}", item.text),
                })
                .collect();
            Ok(TranslateResponse { items })
        }
    }

    fn dummy_request(text: &str) -> TranslateRequest {
        TranslateRequest {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            items: vec![BatchItem {
                id: 0,
                text: text.to_string(),
                context_before: vec![],
                context_after: vec![],
            }],
            preserve_markdown: true,
            system_prompt: None,
            user_prompt: None,
        }
    }

    #[tokio::test]
    async fn concurrent_dedup_collapses_identical_requests() {
        let inner = Arc::new(CountingBackend {
            provider_name: "test",
            fingerprint: "fp1".to_string(),
            call_count: AtomicUsize::new(0),
        });
        let backend = Arc::new(DeduplicatingBackend::new(inner.clone()));

        let request = dummy_request("hello");

        let mut handles = Vec::new();
        for _ in 0..100 {
            let b = backend.clone();
            let r = request.clone();
            handles.push(tokio::spawn(async move { b.translate_batch(r).await }));
        }

        let results = futures::future::join_all(handles).await;
        for result in results {
            let response = result.unwrap().unwrap();
            assert_eq!(response.items.len(), 1);
            assert_eq!(response.items[0].text, "translated: hello");
        }

        assert_eq!(inner.call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn distinct_texts_do_not_dedup() {
        let inner = Arc::new(CountingBackend {
            provider_name: "test",
            fingerprint: "fp1".to_string(),
            call_count: AtomicUsize::new(0),
        });
        let backend = Arc::new(DeduplicatingBackend::new(inner.clone()));

        let req1 = dummy_request("hello");
        let req2 = dummy_request("world");

        let mut handles = Vec::new();
        for _ in 0..10 {
            let b = backend.clone();
            let r = req1.clone();
            handles.push(tokio::spawn(async move { b.translate_batch(r).await }));
            let b = backend.clone();
            let r = req2.clone();
            handles.push(tokio::spawn(async move { b.translate_batch(r).await }));
        }

        let results = futures::future::join_all(handles).await;
        for r in results {
            r.unwrap().unwrap();
        }

        assert_eq!(inner.call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn different_target_lang_does_not_dedup() {
        let inner = Arc::new(CountingBackend {
            provider_name: "test",
            fingerprint: "fp1".to_string(),
            call_count: AtomicUsize::new(0),
        });
        let backend = Arc::new(DeduplicatingBackend::new(inner.clone()));

        let mut handles = Vec::new();
        for target in ["zh", "ja", "de"] {
            let req = TranslateRequest {
                target_lang: target.to_string(),
                ..dummy_request("hello")
            };
            for _ in 0..5 {
                let b = backend.clone();
                let r = req.clone();
                handles.push(tokio::spawn(async move { b.translate_batch(r).await }));
            }
        }

        let results = futures::future::join_all(handles).await;
        for r in results {
            r.unwrap().unwrap();
        }

        assert_eq!(inner.call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn different_provider_fingerprint_does_not_dedup() {
        let inner1 = Arc::new(CountingBackend {
            provider_name: "test",
            fingerprint: "fp-a".to_string(),
            call_count: AtomicUsize::new(0),
        });
        let inner2 = Arc::new(CountingBackend {
            provider_name: "test",
            fingerprint: "fp-b".to_string(),
            call_count: AtomicUsize::new(0),
        });
        let backend1 = Arc::new(DeduplicatingBackend::new(inner1.clone()));
        let backend2 = Arc::new(DeduplicatingBackend::new(inner2.clone()));

        let request = dummy_request("hello");

        let mut handles = Vec::new();
        for _ in 0..10 {
            let b = backend1.clone();
            let r = request.clone();
            handles.push(tokio::spawn(async move { b.translate_batch(r).await }));
            let b = backend2.clone();
            let r = request.clone();
            handles.push(tokio::spawn(async move { b.translate_batch(r).await }));
        }

        let results = futures::future::join_all(handles).await;
        for r in results {
            r.unwrap().unwrap();
        }

        assert_eq!(inner1.call_count.load(Ordering::SeqCst), 1);
        assert_eq!(inner2.call_count.load(Ordering::SeqCst), 1);
    }
}
