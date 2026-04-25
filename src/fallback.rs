use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use rand::Rng;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::dedup::DeduplicatingBackend;
use crate::error::{MdTranslatorError, Result};
use crate::provider::TranslationBackend;
use crate::types::{RuntimeOptions, TranslateRequest, TranslateResponse};

/// Record of a provider fallback event, capturing which provider failed and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackDecision {
    pub provider: String,
    pub reason: String,
}

/// Backend that chains multiple providers with automatic per-document fallback.
///
/// When a provider exhausts retries for a document, `MultiProviderBackend` restarts
/// translation of the entire document with the next provider in the chain.
#[derive(Clone)]
pub struct MultiProviderBackend {
    providers: Vec<Arc<dyn TranslationBackend>>,
    decisions: Arc<Mutex<Vec<FallbackDecision>>>,
}

impl MultiProviderBackend {
    pub fn new(providers: Vec<Arc<dyn TranslationBackend>>) -> Self {
        Self {
            providers,
            decisions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_deduplicated_providers(&self) -> Self {
        Self {
            providers: self
                .providers
                .iter()
                .cloned()
                .map(|provider| {
                    Arc::new(DeduplicatingBackend::new(provider)) as Arc<dyn TranslationBackend>
                })
                .collect(),
            decisions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn providers(&self) -> &[Arc<dyn TranslationBackend>] {
        &self.providers
    }

    pub fn decisions(&self) -> Vec<FallbackDecision> {
        self.decisions
            .lock()
            .expect("fallback decision mutex poisoned")
            .clone()
    }

    pub fn clear_decisions(&self) {
        self.decisions
            .lock()
            .expect("fallback decision mutex poisoned")
            .clear();
    }

    fn push_decision(&self, provider: &dyn TranslationBackend, reason: impl Into<String>) {
        self.decisions
            .lock()
            .expect("fallback decision mutex poisoned")
            .push(FallbackDecision {
                provider: provider.name().to_string(),
                reason: reason.into(),
            });
    }

    pub async fn translate_document_requests(
        &self,
        requests: &[TranslateRequest],
        runtime: &RuntimeOptions,
    ) -> Result<Vec<TranslateResponse>> {
        self.clear_decisions();

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let mut failure_reasons = Vec::new();

        for provider in &self.providers {
            let mut responses = Vec::with_capacity(requests.len());
            let mut restart_document = false;

            for request in requests {
                match translate_with_retries(provider.clone(), request.clone(), runtime).await {
                    Ok(response) => responses.push(response),
                    Err(err) if is_retryable_error(&err) => {
                        let reason = err.to_string();
                        self.push_decision(provider.as_ref(), reason.clone());
                        warn!(
                            provider = provider.name(),
                            error = %err,
                            "provider exhausted retries, restarting document with fallback provider"
                        );
                        failure_reasons.push(format!("{}: {}", provider.name(), reason));
                        restart_document = true;
                        break;
                    }
                    Err(err) => return Err(err),
                }
            }

            if !restart_document {
                debug!(
                    provider = provider.name(),
                    "document translated with selected provider"
                );
                return Ok(responses);
            }
        }

        Err(MdTranslatorError::Provider(format!(
            "all providers failed after retry exhaustion: {}",
            failure_reasons.join(" | ")
        )))
    }
}

#[async_trait]
impl TranslationBackend for MultiProviderBackend {
    fn name(&self) -> &'static str {
        "multi-provider"
    }

    fn cache_fingerprint(&self) -> String {
        self.providers
            .iter()
            .map(|provider| format!("{}({})", provider.name(), provider.cache_fingerprint()))
            .collect::<Vec<_>>()
            .join("->")
    }

    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
        self.clear_decisions();

        let runtime = RuntimeOptions::default();
        let responses = self
            .translate_document_requests(&[request], &runtime)
            .await?;

        responses.into_iter().next().ok_or_else(|| {
            MdTranslatorError::Provider(
                "multi-provider backend unexpectedly produced no batch response".to_string(),
            )
        })
    }
}

fn is_retryable_error(err: &MdTranslatorError) -> bool {
    match err {
        MdTranslatorError::Http(http_err) => {
            http_err.is_timeout()
                || http_err.is_connect()
                || http_err
                    .status()
                    .map(|s| s.as_u16() == 429 || s.is_server_error())
                    .unwrap_or(false)
        }
        MdTranslatorError::Provider(message) => {
            message.contains("status 429")
                || message.contains("status 500")
                || message.contains("status 502")
                || message.contains("status 503")
                || message.contains("status 504")
                || message.to_lowercase().contains("timeout")
        }
        MdTranslatorError::ProviderHttp { status_code, .. } => {
            *status_code == 429 || (500..=504).contains(status_code)
        }
        MdTranslatorError::Io(_) => false,
        MdTranslatorError::Json(_) => false,
        MdTranslatorError::Utf8(_) => false,
        MdTranslatorError::Config(_) => false,
        MdTranslatorError::InvalidResponse(_) => false,
        MdTranslatorError::BatchMismatch { .. } => false,
        MdTranslatorError::Join(_) => false,
    }
}

async fn translate_with_retries(
    backend: Arc<dyn TranslationBackend>,
    request: TranslateRequest,
    runtime: &RuntimeOptions,
) -> Result<TranslateResponse> {
    let mut attempt = 0u32;
    let mut backoff_ms = runtime.retry_backoff_ms.max(1);

    loop {
        let started_at = std::time::Instant::now();
        let result = timeout(
            Duration::from_secs(runtime.request_timeout_secs),
            backend.translate_batch(request.clone()),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                debug!(
                    backend = backend.name(),
                    attempt,
                    latency_ms = started_at.elapsed().as_millis(),
                    "batch translated"
                );
                return Ok(response);
            }
            Ok(Err(err)) => {
                if !is_retryable_error(&err) {
                    return Err(err);
                }
                if attempt >= runtime.max_retries {
                    return Err(err);
                }
                warn!(backend = backend.name(), attempt, error = %err, "batch translation failed, retrying");
            }
            Err(_) => {
                if attempt >= runtime.max_retries {
                    return Err(MdTranslatorError::Provider(format!(
                        "request timeout after {} seconds",
                        runtime.request_timeout_secs
                    )));
                }
                warn!(
                    backend = backend.name(),
                    attempt, "batch translation timed out, retrying"
                );
            }
        }

        attempt += 1;
        let jitter = rand::rng().random_range(0..=(backoff_ms / 5).max(1));
        tokio::time::sleep(Duration::from_millis(backoff_ms + jitter)).await;
        backoff_ms = (backoff_ms * 2).min(10_000);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::provider::MAX_RETRY_AFTER_MS;
    use crate::types::{BatchItem, TranslateResponseItem};

    struct ScriptedBackend {
        name: &'static str,
        fingerprint: &'static str,
        calls: AtomicUsize,
        responses: Mutex<VecDeque<Result<TranslateResponse>>>,
    }

    impl ScriptedBackend {
        fn new(name: &'static str, responses: Vec<Result<TranslateResponse>>) -> Self {
            Self {
                name,
                fingerprint: "fp",
                calls: AtomicUsize::new(0),
                responses: Mutex::new(responses.into()),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl TranslationBackend for ScriptedBackend {
        fn name(&self) -> &'static str {
            self.name
        }

        fn cache_fingerprint(&self) -> String {
            self.fingerprint.to_string()
        }

        async fn translate_batch(&self, _request: TranslateRequest) -> Result<TranslateResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.responses
                .lock()
                .expect("scripted backend mutex poisoned")
                .pop_front()
                .unwrap_or_else(|| {
                    Err(MdTranslatorError::Provider(format!(
                        "{} missing scripted response",
                        self.name
                    )))
                })
        }
    }

    fn request(id: usize, text: &str) -> TranslateRequest {
        TranslateRequest {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            items: vec![BatchItem {
                id,
                text: text.to_string(),
                context_before: vec![],
                context_after: vec![],
            }],
            preserve_markdown: true,
            system_prompt: None,
            user_prompt: None,
        }
    }

    fn response(id: usize, text: &str) -> Result<TranslateResponse> {
        Ok(TranslateResponse {
            items: vec![TranslateResponseItem {
                id,
                text: text.to_string(),
            }],
        })
    }

    #[tokio::test]
    async fn provider_fallback() {
        let primary = Arc::new(ScriptedBackend::new(
            "primary",
            vec![
                response(0, "alpha-primary"),
                Err(MdTranslatorError::Provider(
                    "status 503 upstream overloaded".to_string(),
                )),
            ],
        ));
        let secondary = Arc::new(ScriptedBackend::new(
            "secondary",
            vec![
                response(0, "alpha-secondary"),
                response(1, "beta-secondary"),
            ],
        ));

        let backend = MultiProviderBackend::new(vec![primary.clone(), secondary.clone()]);
        let requests = vec![request(0, "alpha"), request(1, "beta")];
        let responses = backend
            .translate_document_requests(
                &requests,
                &RuntimeOptions {
                    max_retries: 0,
                    retry_backoff_ms: 1,
                    request_timeout_secs: 1,
                    ..RuntimeOptions::default()
                },
            )
            .await
            .unwrap();

        assert_eq!(primary.calls(), 2);
        assert_eq!(secondary.calls(), 2);
        assert_eq!(responses.len(), 2);
        assert_eq!(responses[0].items[0].text, "alpha-secondary");
        assert_eq!(responses[1].items[0].text, "beta-secondary");
        assert_eq!(
            backend.decisions(),
            vec![FallbackDecision {
                provider: "primary".to_string(),
                reason: "provider error: status 503 upstream overloaded".to_string(),
            }]
        );
    }

    #[tokio::test]
    async fn all_failures_are_aggregated() {
        let primary = Arc::new(ScriptedBackend::new(
            "primary",
            vec![Err(MdTranslatorError::Provider(
                "status 429 rate limited".to_string(),
            ))],
        ));
        let secondary = Arc::new(ScriptedBackend::new(
            "secondary",
            vec![Err(MdTranslatorError::Provider(
                "request timeout after 1 seconds".to_string(),
            ))],
        ));

        let backend = MultiProviderBackend::new(vec![primary, secondary]);
        let err = backend
            .translate_document_requests(
                &[request(0, "alpha")],
                &RuntimeOptions {
                    max_retries: 0,
                    retry_backoff_ms: 1,
                    request_timeout_secs: 1,
                    ..RuntimeOptions::default()
                },
            )
            .await
            .unwrap_err();

        let message = err.to_string();
        assert!(message.contains("all providers failed after retry exhaustion"));
        assert!(message.contains("primary: provider error: status 429 rate limited"));
        assert!(message.contains("secondary: provider error: request timeout after 1 seconds"));
        assert_eq!(backend.decisions().len(), 2);
    }

    #[tokio::test]
    async fn retry_after_hint_is_capped_for_fallback_retries() {
        let backend = Arc::new(ScriptedBackend::new(
            "primary",
            vec![
                Err(MdTranslatorError::ProviderHttp {
                    provider: "primary",
                    status_code: 429,
                    retry_after_ms: Some(MAX_RETRY_AFTER_MS * 10),
                    message: "slow down".to_string(),
                }),
                response(0, "ok"),
            ],
        ));

        let started_at = std::time::Instant::now();
        let response = translate_with_retries(
            backend.clone(),
            request(0, "alpha"),
            &RuntimeOptions {
                max_retries: 1,
                retry_backoff_ms: 1,
                request_timeout_secs: 1,
                ..RuntimeOptions::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(response.items[0].text, "ok");
        assert_eq!(backend.calls(), 2);
        assert!(started_at.elapsed() < Duration::from_secs(10));
    }
}
