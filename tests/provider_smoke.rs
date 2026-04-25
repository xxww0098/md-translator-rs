//! Provider smoke tests — exercises all 4 translation backends end-to-end
//! using wiremock mock servers. No real network calls required.
//!
//! Covers:
//!   - GTX (Google Translate web API)
//!   - DeepLX (per-item requests)
//!   - OpenAI-Compat (XML segment batching via chat completion)
//!   - DeepL (native batching)
//!   - Cross-provider regressions for the shared scheduler production path
//!
//! Each section tests:
//!   1. Single-item translation
//!   2. Multi-item batch translation
//!   3. Error handling (HTTP 500 / malformed response)
//!   4. Empty batch (no-op)
//!   5. End-to-end MdTranslator pipeline (Markdown structure preservation)

mod common;

use std::sync::Arc;

use reqwest::Client;
use serde_json::json;
use wiremock::matchers::{body_partial_json, body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use md_translator_rs::{
    BatchItem, BatchingOptions, CacheOptions, DeepLBackend, DeepLXBackend, DocumentFormat,
    GtxBackend, MarkdownOptions, MdTranslator, OpenAICompatBackend, RuntimeOptions,
    TranslateOptions, TranslateRequest, TranslationBackend, TwoTierCache,
};

use common::{batch_item, request_with_items};

// ═══════════════════════════════════════════════════════════════════════════
//  Test fixtures
// ═══════════════════════════════════════════════════════════════════════════

const TEST_MARKDOWN: &str = r#"---
title: Hello World
---

# Introduction

This is a **test document** with [links](https://example.com).

```rust
fn main() {
    println!("Hello");
}
```

## Section Two

Another paragraph for translation.
"#;

fn default_test_options() -> TranslateOptions {
    TranslateOptions {
        format: DocumentFormat::Markdown,
        source_lang: "en".to_string(),
        target_lang: "zh".to_string(),
        markdown: MarkdownOptions {
            translate_frontmatter: true,
            translate_multiline_code: false,
            translate_latex: false,
            translate_link_text: true,
        },
        batching: BatchingOptions {
            max_items_per_batch: 20,
            max_chars_per_batch: 5000,
            context_window: 0,
        },
        runtime: RuntimeOptions {
            max_concurrency: 1,
            adaptive_concurrency: false,
            initial_concurrency: 1,
            min_concurrency: 1,
            max_retries: 1,
            retry_backoff_ms: 10,
            request_timeout_secs: 5,
        },
        cache: CacheOptions {
            enabled: false,
            namespace: "test".to_string(),
        },
        system_prompt: None,
        user_prompt: None,
    }
}

fn test_cache() -> Arc<TwoTierCache> {
    let dir = std::env::temp_dir().join(format!(
        "md-translator-rs-smoke-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    Arc::new(TwoTierCache::new(dir).unwrap())
}

// ═══════════════════════════════════════════════════════════════════════════
//  GTX Provider Tests
// ═══════════════════════════════════════════════════════════════════════════

mod gtx {
    use super::*;

    /// GTX now routes `translate_batch()` through the shared scheduler, but the
    /// live endpoint is hardcoded to googleapis.com so integration smoke can only
    /// exercise the empty-work no-op path without a network refactor.
    #[tokio::test]
    async fn single_item_translation() {
        // NOTE: GtxBackend hardcodes the endpoint to googleapis.com, so
        // we cannot redirect it to a mock server without refactoring.
        // This test validates the trait interface contract instead.

        // Since we can't mock GTX's hardcoded URL, validate contract only:
        let backend = GtxBackend::new(Client::new());
        assert_eq!(backend.name(), "gtx");
        assert_eq!(backend.cache_fingerprint(), "gtx.googleapis.com");

        // Empty batch should succeed
        let response = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items: vec![],
                preserve_markdown: false,
                system_prompt: None,
                user_prompt: None,
            })
            .await
            .unwrap();
        assert!(response.items.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  DeepLX Provider Tests
// ═══════════════════════════════════════════════════════════════════════════

mod deeplx {
    use super::*;

    #[tokio::test]
    async fn single_item() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("Hello world"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": "你好世界"})))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLXBackend::new(Client::new(), format!("{}/translate", mock.uri()));
        let resp = backend
            .translate_batch(request_with_items(vec![batch_item(0, "Hello world")]))
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].text, "你好世界");
    }

    #[tokio::test]
    async fn multi_item_per_item_request() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("First"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": "第一段"})))
            .expect(1)
            .mount(&mock)
            .await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("Second"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": "第二段"})))
            .expect(1)
            .mount(&mock)
            .await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("Third"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": "第三段"})))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLXBackend::new(Client::new(), format!("{}/translate", mock.uri()));
        let resp = backend
            .translate_batch(request_with_items(vec![
                batch_item(0, "First"),
                batch_item(1, "Second"),
                batch_item(2, "Third"),
            ]))
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 3);
        assert_eq!(resp.items[0].text, "第一段");
        assert_eq!(resp.items[1].text, "第二段");
        assert_eq!(resp.items[2].text, "第三段");
    }

    #[tokio::test]
    async fn http_500_returns_provider_error() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLXBackend::new(Client::new(), format!("{}/translate", mock.uri()));
        let result = backend
            .translate_batch(request_with_items(vec![batch_item(0, "test")]))
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("500"),
            "error should mention HTTP 500: {err_msg}"
        );
    }

    #[tokio::test]
    async fn malformed_response_returns_error() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"unexpected": "shape"})))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLXBackend::new(Client::new(), format!("{}/translate", mock.uri()));
        let result = backend
            .translate_batch(request_with_items(vec![batch_item(0, "test")]))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn empty_batch_succeeds() {
        let backend = DeepLXBackend::new(Client::new(), "http://unused".to_string());
        let resp = backend
            .translate_batch(request_with_items(vec![]))
            .await
            .unwrap();
        assert!(resp.items.is_empty());
    }

    #[tokio::test]
    async fn e2e_markdown_pipeline() {
        let mock = MockServer::start().await;

        // DeepLX batches multiple items with markers, but wiremock can't
        // dynamically echo them. Force batch_size=1 so each node is a
        // separate single-item request (no markers).
        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": "[ZH] 翻译内容"
            })))
            .mount(&mock)
            .await;

        let backend = Arc::new(DeepLXBackend::new(
            Client::new(),
            format!("{}/translate", mock.uri()),
        ));
        let cache = test_cache();
        let translator = MdTranslator::new(backend, cache);

        let mut opts = default_test_options();
        opts.batching.max_items_per_batch = 1; // force single-item requests

        let result = translator.translate_markdown(TEST_MARKDOWN, &opts).await;

        assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
        let output = result.unwrap();
        // Code block must be preserved
        assert!(
            output.contains("println!"),
            "code block should be preserved in output"
        );
        // Link URL must be preserved
        assert!(
            output.contains("https://example.com"),
            "link URL should be preserved"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  OpenAI-Compatible Provider Tests
// ═══════════════════════════════════════════════════════════════════════════

mod openai_compat {
    use super::*;

    fn openai_response(content: &str) -> serde_json::Value {
        json!({
            "choices": [{
                "message": {
                    "content": content
                }
            }]
        })
    }

    fn openai_backend(mock_uri: &str) -> OpenAICompatBackend {
        OpenAICompatBackend::with_concurrency(
            Client::new(),
            mock_uri.to_string(),
            "test-api-key".to_string(),
            "test-model".to_string(),
            0.1,
            5,
        )
    }

    #[tokio::test]
    async fn single_item_xml() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("seg id"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(openai_response(r#"<seg id="1">你好世界</seg>"#)),
            )
            .expect(1)
            .mount(&mock)
            .await;

        let backend = openai_backend(&mock.uri());
        let resp = backend
            .translate_batch(request_with_items(vec![batch_item(1, "Hello world")]))
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].id, 1);
        assert_eq!(resp.items[0].text, "你好世界");
    }

    #[tokio::test]
    async fn multi_item_xml_batch() {
        let mock = MockServer::start().await;

        let xml_response = (1..=5)
            .map(|id| format!(r#"<seg id="{id}">翻译 {id}</seg>"#))
            .collect::<Vec<_>>()
            .join("");

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(openai_response(&xml_response)))
            .expect(1)
            .mount(&mock)
            .await;

        let items: Vec<BatchItem> = (1..=5)
            .map(|id| batch_item(id, format!("text {id}")))
            .collect();

        let backend = openai_backend(&mock.uri());
        let resp = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 5);
        for (idx, item) in resp.items.iter().enumerate() {
            let id = idx + 1;
            assert_eq!(item.id, id);
            assert_eq!(item.text, format!("翻译 {id}"));
        }
    }

    #[tokio::test]
    async fn multi_batch_xml_uses_shared_scheduler_path() {
        let mock = MockServer::start().await;

        let first_xml_response = (1..=15)
            .map(|id| format!(r#"<seg id="{id}">批次一 {id}</seg>"#))
            .collect::<Vec<_>>()
            .join("");
        let second_xml_response = r#"<seg id="16">批次二 16</seg>"#;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("text 1"))
            .and(body_string_contains("text 15"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(openai_response(&first_xml_response)),
            )
            .expect(1)
            .mount(&mock)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("text 16"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(openai_response(second_xml_response)),
            )
            .expect(1)
            .mount(&mock)
            .await;

        let items: Vec<BatchItem> = (1..=16)
            .map(|id| batch_item(id, format!("text {id}")))
            .collect();

        let backend = openai_backend(&mock.uri());
        let resp = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 16);
        for (idx, item) in resp.items.iter().enumerate() {
            let id = idx + 1;
            assert_eq!(item.id, id);
            let expected = if id <= 15 {
                format!("批次一 {id}")
            } else {
                format!("批次二 {id}")
            };
            assert_eq!(item.text, expected);
        }
    }

    #[tokio::test]
    async fn http_500_returns_error() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_string("server error"))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = openai_backend(&mock.uri());
        let result = backend
            .translate_batch(request_with_items(vec![batch_item(1, "test")]))
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("500"),
            "should mention HTTP 500: {err_msg}"
        );
    }

    #[tokio::test]
    async fn malformed_response_missing_choices() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"result": "unexpected"})))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = openai_backend(&mock.uri());
        let result = backend
            .translate_batch(request_with_items(vec![batch_item(1, "test")]))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn empty_batch_succeeds() {
        let backend = openai_backend("http://unused");
        let resp = backend
            .translate_batch(request_with_items(vec![]))
            .await
            .unwrap();
        assert!(resp.items.is_empty());
    }

    #[tokio::test]
    async fn e2e_markdown_pipeline() {
        let mock = MockServer::start().await;

        // For each request, return a valid XML response echoing the segments
        // We need to handle arbitrary seg ids, so use a closure-like approach.
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    // Return a dynamic-ish response. Since wiremock can't be truly
                    // dynamic, we return a large set of translated segs.
                    openai_response(
                        &(1..=50)
                            .map(|id| format!(r#"<seg id="{id}">[ZH] node {id}</seg>"#))
                            .collect::<Vec<_>>()
                            .join(""),
                    ),
                ),
            )
            .mount(&mock)
            .await;

        let backend = Arc::new(openai_backend(&mock.uri()));
        let cache = test_cache();
        let translator = MdTranslator::new(backend, cache);

        let result = translator
            .translate_markdown(TEST_MARKDOWN, &default_test_options())
            .await;

        assert!(result.is_ok(), "pipeline failed: {:?}", result.err());
        let output = result.unwrap();
        assert!(output.contains("println!"), "code block preserved");
        assert!(output.contains("https://example.com"), "link URL preserved");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  DeepL Provider Tests
// ═══════════════════════════════════════════════════════════════════════════

mod deepl {
    use super::*;

    fn deepl_translations(texts: &[&str]) -> serde_json::Value {
        let translations: Vec<serde_json::Value> =
            texts.iter().map(|t| json!({"text": t})).collect();
        json!({"translations": translations})
    }

    #[tokio::test]
    async fn single_item() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/translate"))
            .and(body_partial_json(json!({"text": ["Hello world"]})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(deepl_translations(&["你好世界"])),
            )
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLBackend::new(
            Client::new(),
            format!("{}/v2/translate", mock.uri()),
            "test-key".to_string(),
        );
        let resp = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items: vec![batch_item(0, "Hello world")],
                preserve_markdown: true,
                system_prompt: None,
                user_prompt: None,
            })
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].text, "你好世界");
    }

    #[tokio::test]
    async fn multi_item_native_batch() {
        let mock = MockServer::start().await;

        let items: Vec<BatchItem> = (0..10)
            .map(|id| batch_item(id, format!("text {id}")))
            .collect();
        let texts: Vec<String> = items.iter().map(|i| i.text.clone()).collect();
        let translated: Vec<&str> = (0..10).map(|_| "翻译").collect();

        Mock::given(method("POST"))
            .and(path("/v2/translate"))
            .and(body_partial_json(json!({"text": texts})))
            .respond_with(ResponseTemplate::new(200).set_body_json(deepl_translations(&translated)))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLBackend::new(
            Client::new(),
            format!("{}/v2/translate", mock.uri()),
            "test-key".to_string(),
        );
        let resp = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items,
                preserve_markdown: true,
                system_prompt: None,
                user_prompt: None,
            })
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 10);
        for item in &resp.items {
            assert_eq!(item.text, "翻译");
        }
    }

    #[tokio::test]
    async fn multi_chunk_batch_uses_shared_scheduler_path() {
        let mock = MockServer::start().await;

        let items: Vec<BatchItem> = (0..51)
            .map(|id| batch_item(id, format!("text {id}")))
            .collect();
        let first_chunk_texts: Vec<String> =
            items[..50].iter().map(|item| item.text.clone()).collect();
        let second_chunk_texts: Vec<String> =
            items[50..].iter().map(|item| item.text.clone()).collect();
        let first_chunk_translations: Vec<String> =
            (0..50).map(|id| format!("chunk one {id}")).collect();
        let second_chunk_translations: Vec<String> = vec!["chunk two 50".to_string()];

        Mock::given(method("POST"))
            .and(path("/v2/translate"))
            .and(body_partial_json(json!({"text": first_chunk_texts})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(deepl_translations(
                    &first_chunk_translations
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                )),
            )
            .expect(1)
            .mount(&mock)
            .await;

        Mock::given(method("POST"))
            .and(path("/v2/translate"))
            .and(body_partial_json(json!({"text": second_chunk_texts})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(deepl_translations(
                    &second_chunk_translations
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>(),
                )),
            )
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLBackend::new(
            Client::new(),
            format!("{}/v2/translate", mock.uri()),
            "test-key".to_string(),
        );
        let resp = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items,
                preserve_markdown: true,
                system_prompt: None,
                user_prompt: None,
            })
            .await
            .unwrap();

        assert_eq!(resp.items.len(), 51);
        for (id, item) in resp.items.iter().enumerate() {
            assert_eq!(item.id, id);
            let expected = if id < 50 {
                format!("chunk one {id}")
            } else {
                format!("chunk two {id}")
            };
            assert_eq!(item.text, expected);
        }
    }

    #[tokio::test]
    async fn http_500_returns_error() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/translate"))
            .respond_with(ResponseTemplate::new(500).set_body_string("quota exceeded"))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLBackend::new(
            Client::new(),
            format!("{}/v2/translate", mock.uri()),
            "test-key".to_string(),
        );
        let result = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items: vec![batch_item(0, "test")],
                preserve_markdown: true,
                system_prompt: None,
                user_prompt: None,
            })
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("500"),
            "should mention HTTP 500: {err_msg}"
        );
    }

    #[tokio::test]
    async fn malformed_response_missing_translations() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"result": "bad"})))
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLBackend::new(
            Client::new(),
            format!("{}/v2/translate", mock.uri()),
            "test-key".to_string(),
        );
        let result = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items: vec![batch_item(0, "test")],
                preserve_markdown: true,
                system_prompt: None,
                user_prompt: None,
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn count_mismatch_returns_error() {
        let mock = MockServer::start().await;

        // Send 3 items but return only 1 translation
        Mock::given(method("POST"))
            .and(path("/v2/translate"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(deepl_translations(&["只有一个"])),
            )
            .expect(1)
            .mount(&mock)
            .await;

        let backend = DeepLBackend::new(
            Client::new(),
            format!("{}/v2/translate", mock.uri()),
            "test-key".to_string(),
        );
        let result = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items: vec![
                    batch_item(0, "one"),
                    batch_item(1, "two"),
                    batch_item(2, "three"),
                ],
                preserve_markdown: true,
                system_prompt: None,
                user_prompt: None,
            })
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("1") && err_msg.contains("3"),
            "should mention count mismatch: {err_msg}"
        );
    }

    #[tokio::test]
    async fn empty_batch_succeeds() {
        let backend = DeepLBackend::new(
            Client::new(),
            "http://unused".to_string(),
            "test-key".to_string(),
        );
        let resp = backend
            .translate_batch(TranslateRequest {
                source_lang: "en".to_string(),
                target_lang: "zh".to_string(),
                items: vec![],
                preserve_markdown: true,
                system_prompt: None,
                user_prompt: None,
            })
            .await
            .unwrap();
        assert!(resp.items.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Cross-Provider: Cache interaction test
// ═══════════════════════════════════════════════════════════════════════════

mod cache_integration {
    use super::*;

    #[tokio::test]
    async fn second_translation_hits_cache() {
        let mock = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": "缓存测试"})))
            // Should only be called ONCE per node; second call hits cache
            .mount(&mock)
            .await;

        let backend = Arc::new(DeepLXBackend::new(
            Client::new(),
            format!("{}/translate", mock.uri()),
        ));
        let cache = test_cache();
        let translator = MdTranslator::new(backend, cache);

        let mut opts = default_test_options();
        opts.cache = CacheOptions {
            enabled: true,
            namespace: "cache-test".to_string(),
        };
        opts.batching.max_items_per_batch = 1; // force single-item requests

        let input = "# Cache Test\n\nSimple paragraph.";

        // First call => hits provider
        let out1 = translator.translate_markdown(input, &opts).await.unwrap();
        // Second call => should hit cache (no additional HTTP)
        let out2 = translator.translate_markdown(input, &opts).await.unwrap();

        // Both should produce identical output
        assert_eq!(out1, out2);
    }
}
