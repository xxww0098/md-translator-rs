use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use md_translator_rs::{
    BatchItem, CacheOptions, DocumentFormat, MarkdownOptions, MdTranslator, MdTranslatorError,
    MultiProviderBackend, RuntimeOptions, TranslateOptions, TranslateRequest, TranslateResponse,
    TranslateResponseItem, TranslationBackend, TwoTierCache,
};

struct MockBackend {
    name: &'static str,
    call_count: AtomicUsize,
    fail_after: Option<usize>,
    fail_message: String,
    prefix: String,
}

impl MockBackend {
    fn new(name: &'static str, prefix: &str) -> Self {
        Self {
            name,
            call_count: AtomicUsize::new(0),
            fail_after: None,
            fail_message: String::new(),
            prefix: prefix.to_string(),
        }
    }

    fn failing(name: &'static str, fail_after: usize, message: &str) -> Self {
        Self {
            name,
            call_count: AtomicUsize::new(0),
            fail_after: Some(fail_after),
            fail_message: message.to_string(),
            prefix: String::new(),
        }
    }

    fn calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl TranslationBackend for MockBackend {
    fn name(&self) -> &'static str {
        self.name
    }

    fn cache_fingerprint(&self) -> String {
        "fp".to_string()
    }

    async fn translate_batch(
        &self,
        request: TranslateRequest,
    ) -> md_translator_rs::Result<TranslateResponse> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        if let Some(fail_after) = self.fail_after
            && count >= fail_after
        {
            return Err(MdTranslatorError::Provider(self.fail_message.clone()));
        }
        Ok(TranslateResponse {
            items: request
                .items
                .into_iter()
                .map(|item| TranslateResponseItem {
                    id: item.id,
                    text: format!("{}{}", self.prefix, item.text),
                })
                .collect(),
        })
    }
}

fn plain_text_options() -> TranslateOptions {
    TranslateOptions {
        format: DocumentFormat::PlainText,
        source_lang: "en".to_string(),
        target_lang: "zh".to_string(),
        markdown: MarkdownOptions::default(),
        batching: md_translator_rs::BatchingOptions {
            max_items_per_batch: 2,
            max_chars_per_batch: 1000,
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
            enabled: false,
            namespace: "test".to_string(),
        },
        system_prompt: None,
        user_prompt: None,
    }
}

#[tokio::test]
async fn integration_fallback_primary_fails_backup_succeeds() {
    let primary = Arc::new(MockBackend::failing("primary", 1, "status 503 overloaded"));
    let secondary = Arc::new(MockBackend::new("secondary", "[backup] "));

    let multi = MultiProviderBackend::new(vec![primary.clone(), secondary.clone()]);
    let cache_dir = std::env::temp_dir().join(format!("mdt-fb-1-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(Arc::new(multi), cache);

    let input = "alpha\nbeta\ngamma";
    let options = plain_text_options();
    let output = translator
        .translate_plain_text(input, &options)
        .await
        .unwrap();

    assert!(output.contains("[backup] alpha"));
    assert!(output.contains("[backup] beta"));
    assert!(output.contains("[backup] gamma"));
    assert!(!output.contains("[primary]"));

    assert_eq!(primary.calls(), 2);
    assert_eq!(secondary.calls(), 2);
}

#[tokio::test]
async fn integration_fallback_all_providers_fail() {
    let primary = Arc::new(MockBackend::failing(
        "primary",
        0,
        "status 429 rate limited",
    ));
    let secondary = Arc::new(MockBackend::failing(
        "secondary",
        0,
        "status 500 internal error",
    ));

    let multi = MultiProviderBackend::new(vec![primary, secondary]);
    let cache_dir = std::env::temp_dir().join(format!("mdt-fb-2-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(Arc::new(multi), cache);

    let input = "hello";
    let options = plain_text_options();
    let err = translator
        .translate_plain_text(input, &options)
        .await
        .unwrap_err();

    let msg = err.to_string();
    assert!(msg.contains("all providers failed after retry exhaustion"));
    assert!(msg.contains("primary"));
    assert!(msg.contains("secondary"));
}

#[tokio::test]
async fn integration_fallback_single_batch_document() {
    let primary = Arc::new(MockBackend::failing("primary", 0, "timeout"));
    let secondary = Arc::new(MockBackend::new("secondary", "[ok] "));

    let multi = MultiProviderBackend::new(vec![primary.clone(), secondary.clone()]);
    let cache_dir = std::env::temp_dir().join(format!("mdt-fb-3-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(Arc::new(multi), cache);

    let input = "only line";
    let options = plain_text_options();
    let output = translator
        .translate_plain_text(input, &options)
        .await
        .unwrap();

    assert_eq!(output, "[ok] only line");
    assert_eq!(primary.calls(), 1);
    assert_eq!(secondary.calls(), 1);
}

#[tokio::test]
async fn integration_fallback_decisions_recorded_directly() {
    let primary = Arc::new(MockBackend::failing("primary", 0, "timeout"));
    let secondary = Arc::new(MockBackend::new("secondary", "[ok] "));

    let multi = MultiProviderBackend::new(vec![primary, secondary]);
    let requests = vec![TranslateRequest {
        source_lang: "en".to_string(),
        target_lang: "zh".to_string(),
        items: vec![BatchItem {
            id: 0,
            text: "hello".to_string(),
            context_before: vec![],
            context_after: vec![],
        }],
        preserve_markdown: false,
        system_prompt: None,
        user_prompt: None,
    }];

    let runtime = RuntimeOptions {
        max_concurrency: 1,
        adaptive_concurrency: false,
        initial_concurrency: 1,
        min_concurrency: 1,
        max_retries: 0,
        retry_backoff_ms: 1,
        request_timeout_secs: 1,
    };

    let result = multi
        .translate_document_requests(&requests, &runtime)
        .await
        .unwrap();
    assert_eq!(result[0].items[0].text, "[ok] hello");

    let decisions = multi.decisions();
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].provider, "primary");
    assert!(decisions[0].reason.contains("timeout"));
}
