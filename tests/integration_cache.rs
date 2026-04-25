use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use md_translator_rs::{
    CacheOptions, DocumentFormat, MarkdownOptions, MdTranslator, RuntimeOptions, TranslateOptions,
    TranslateRequest, TranslateResponse, TranslateResponseItem, TranslationBackend, TwoTierCache,
};

struct CountingBackend {
    call_count: AtomicUsize,
}

impl CountingBackend {
    fn new() -> Self {
        Self {
            call_count: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl TranslationBackend for CountingBackend {
    fn name(&self) -> &'static str {
        "counting"
    }

    fn cache_fingerprint(&self) -> String {
        "fp".to_string()
    }

    async fn translate_batch(
        &self,
        request: TranslateRequest,
    ) -> md_translator_rs::Result<TranslateResponse> {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(TranslateResponse {
            items: request
                .items
                .into_iter()
                .map(|item| TranslateResponseItem {
                    id: item.id,
                    text: format!("translated: {}", item.text),
                })
                .collect(),
        })
    }
}

fn cache_options() -> TranslateOptions {
    TranslateOptions {
        format: DocumentFormat::PlainText,
        source_lang: "en".to_string(),
        target_lang: "zh".to_string(),
        markdown: MarkdownOptions::default(),
        batching: md_translator_rs::BatchingOptions {
            max_items_per_batch: 10,
            max_chars_per_batch: 5000,
            context_window: 0,
        },
        runtime: RuntimeOptions {
            max_concurrency: 1,
            adaptive_concurrency: false,
            initial_concurrency: 1,
            min_concurrency: 1,
            max_retries: 0,
            retry_backoff_ms: 1,
            request_timeout_secs: 1,
        },
        cache: CacheOptions {
            enabled: true,
            namespace: "test".to_string(),
        },
        system_prompt: None,
        user_prompt: None,
    }
}

#[tokio::test]
async fn integration_cache_memory_hit_avoids_backend_call() {
    let backend = Arc::new(CountingBackend::new());
    let cache_dir = std::env::temp_dir().join(format!("mdt-cache-mem-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(backend.clone(), cache);

    let input = "hello world";
    let options = cache_options();

    let out1 = translator
        .translate_plain_text(input, &options)
        .await
        .unwrap();
    assert_eq!(out1, "translated: hello world");
    assert_eq!(backend.calls(), 1);

    let out2 = translator
        .translate_plain_text(input, &options)
        .await
        .unwrap();
    assert_eq!(out2, "translated: hello world");
    assert_eq!(backend.calls(), 1);
}

#[tokio::test]
async fn integration_cache_disk_survives_restart() {
    let cache_dir = std::env::temp_dir().join(format!("mdt-cache-disk-{}", std::process::id()));
    {
        let backend = Arc::new(CountingBackend::new());
        let cache = Arc::new(TwoTierCache::new(cache_dir.clone()).unwrap());
        let translator = MdTranslator::new(backend.clone(), cache);

        let input = "persistent text";
        let options = cache_options();
        let out = translator
            .translate_plain_text(input, &options)
            .await
            .unwrap();
        assert_eq!(out, "translated: persistent text");
        assert_eq!(backend.calls(), 1);
    }

    let backend = Arc::new(CountingBackend::new());
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(backend.clone(), cache);

    let input = "persistent text";
    let options = cache_options();
    let out = translator
        .translate_plain_text(input, &options)
        .await
        .unwrap();
    assert_eq!(out, "translated: persistent text");
    assert_eq!(backend.calls(), 0);
}

#[tokio::test]
async fn integration_cache_different_text_misses() {
    let backend = Arc::new(CountingBackend::new());
    let cache_dir = std::env::temp_dir().join(format!("mdt-cache-miss-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(backend.clone(), cache);

    let options = cache_options();
    let _ = translator
        .translate_plain_text("text one", &options)
        .await
        .unwrap();
    let _ = translator
        .translate_plain_text("text two", &options)
        .await
        .unwrap();

    assert_eq!(backend.calls(), 2);
}

#[tokio::test]
async fn integration_cache_disabled_bypasses_all() {
    let backend = Arc::new(CountingBackend::new());
    let cache_dir = std::env::temp_dir().join(format!("mdt-cache-off-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(backend.clone(), cache);

    let mut options = cache_options();
    options.cache.enabled = false;

    let _ = translator
        .translate_plain_text("same", &options)
        .await
        .unwrap();
    let _ = translator
        .translate_plain_text("same", &options)
        .await
        .unwrap();

    assert_eq!(backend.calls(), 2);
}
