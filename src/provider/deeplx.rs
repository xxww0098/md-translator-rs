use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use tokio::sync::Semaphore;

use crate::error::{MdTranslatorError, Result};
use crate::provider::concurrent::ConcurrentTranslationBackend;
use crate::provider::{TranslationBackend, provider_http_error};
use crate::types::{BatchItem, TranslateRequest, TranslateResponse, TranslateResponseItem};

const DEFAULT_MAX_CONCURRENCY: usize = 10;

#[derive(Debug, Clone)]
/// DeepLX translation backend with per-item requests.
///
/// DeepLX only accepts a single `text` string per request (no native array
/// batching). Earlier versions tried marker-based concatenation, but DeepL's
/// translation engine treats markers as translatable text and destroys them.
/// This implementation sends one HTTP request per batch item instead.
pub struct DeepLXBackend {
    client: Client,
    endpoint: String,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone)]
pub struct DeepLXWorkUnit {
    item: BatchItem,
    source_lang: String,
    target_lang: String,
}

impl DeepLXBackend {
    /// Creates a DeepLXBackend with the default concurrency limit of 10.
    pub fn new(client: Client, endpoint: String) -> Self {
        Self::with_concurrency(client, endpoint, DEFAULT_MAX_CONCURRENCY)
    }

    /// Creates a DeepLXBackend with a custom concurrency limit.
    pub fn with_concurrency(client: Client, endpoint: String, max_concurrency: usize) -> Self {
        Self {
            client,
            endpoint,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    fn translate_payload(text: &str, source_lang: &str, target_lang: &str) -> serde_json::Value {
        let source_lang = if source_lang.eq_ignore_ascii_case("auto") {
            serde_json::Value::Null
        } else {
            serde_json::Value::String(source_lang.to_uppercase())
        };

        json!({
            "text": text,
            "source_lang": source_lang,
            "target_lang": target_lang.to_uppercase(),
        })
    }

    fn extract_translation(parsed: &serde_json::Value) -> Option<String> {
        parsed
            .get("data")
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .or_else(|| {
                parsed
                    .get("data")
                    .and_then(|v| v.get("text"))
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
            .or_else(|| {
                parsed
                    .get("translations")
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.get("text"))
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
            .or_else(|| {
                parsed
                    .get("translation")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string)
            })
    }

    async fn translate_one(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(&Self::translate_payload(text, source_lang, target_lang))
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(provider_http_error("deeplx", status, &headers, body));
        }

        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        Self::extract_translation(&parsed).ok_or_else(|| {
            MdTranslatorError::InvalidResponse(
                "DeepLX response missing translation payload in data/data.text/translations[0].text/translation"
                    .to_string(),
            )
        })
    }
}

#[async_trait]
impl ConcurrentTranslationBackend for DeepLXBackend {
    type WorkUnit = DeepLXWorkUnit;
    type WorkOutput = TranslateResponseItem;

    fn split_work(&self, request: &TranslateRequest) -> Vec<Self::WorkUnit> {
        request
            .items
            .iter()
            .cloned()
            .map(|item| DeepLXWorkUnit {
                item,
                source_lang: request.source_lang.clone(),
                target_lang: request.target_lang.clone(),
            })
            .collect()
    }

    async fn execute_work_unit(&self, unit: Self::WorkUnit) -> Result<Self::WorkOutput> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| MdTranslatorError::Join("semaphore closed".to_string()))?;
        let translated = self
            .translate_one(&unit.item.text, &unit.source_lang, &unit.target_lang)
            .await?;
        Ok(TranslateResponseItem {
            id: unit.item.id,
            text: translated,
        })
    }

    async fn assemble_response(
        &self,
        _request: &TranslateRequest,
        outputs: Vec<Self::WorkOutput>,
    ) -> Result<TranslateResponse> {
        Ok(TranslateResponse { items: outputs })
    }
}

#[async_trait]
impl TranslationBackend for DeepLXBackend {
    fn name(&self) -> &'static str {
        "deeplx"
    }

    fn cache_fingerprint(&self) -> String {
        format!("endpoint={}", self.endpoint)
    }

    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
        self.execute_concurrently(&request, DEFAULT_MAX_CONCURRENCY)
            .await
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, method, path},
    };

    use super::*;
    use crate::types::BatchItem;

    fn batch_item(id: usize, text: impl Into<String>) -> BatchItem {
        BatchItem {
            id,
            text: text.into(),
            context_before: vec![],
            context_after: vec![],
        }
    }

    fn request_with_items(items: Vec<BatchItem>) -> TranslateRequest {
        TranslateRequest {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            items,
            preserve_markdown: true,
            system_prompt: None,
            user_prompt: None,
        }
    }

    #[tokio::test]
    async fn deeplx_single_item_translates_correctly() {
        let mock_server = MockServer::start().await;

        let item = batch_item(0, "Hello world");

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("Hello world"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": "你好世界"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let backend = DeepLXBackend::with_concurrency(
            client,
            format!("{}/translate", mock_server.uri()),
            DEFAULT_MAX_CONCURRENCY,
        );

        let response = backend
            .translate_batch(request_with_items(vec![item]))
            .await
            .unwrap();

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].id, 0);
        assert_eq!(response.items[0].text, "你好世界");
    }

    #[tokio::test]
    async fn deeplx_multi_item_sends_per_item_request() {
        let mock_server = MockServer::start().await;

        let items = vec![
            batch_item(0, "First text"),
            batch_item(1, "Second text"),
            batch_item(2, "Third text"),
        ];

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("First text"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": "第一个文本"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("Second text"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": "第二个文本"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_string_contains("Third text"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": "第三个文本"
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let backend = DeepLXBackend::with_concurrency(
            client,
            format!("{}/translate", mock_server.uri()),
            DEFAULT_MAX_CONCURRENCY,
        );

        let response = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();

        assert_eq!(response.items.len(), 3);
        assert_eq!(response.items[0].id, 0);
        assert_eq!(response.items[0].text, "第一个文本");
        assert_eq!(response.items[1].id, 1);
        assert_eq!(response.items[1].text, "第二个文本");
        assert_eq!(response.items[2].id, 2);
        assert_eq!(response.items[2].text, "第三个文本");
    }

    #[tokio::test]
    async fn deeplx_empty_batch_returns_empty() {
        let client = Client::new();
        let backend = DeepLXBackend::with_concurrency(
            client,
            "http://localhost:9999/translate".to_string(),
            DEFAULT_MAX_CONCURRENCY,
        );

        let response = backend
            .translate_batch(request_with_items(vec![]))
            .await
            .unwrap();

        assert!(response.items.is_empty());
    }

    #[tokio::test]
    async fn deeplx_extracts_data_text_format() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": { "text": "嵌套文本" }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let backend = DeepLXBackend::with_concurrency(
            client,
            format!("{}/translate", mock_server.uri()),
            DEFAULT_MAX_CONCURRENCY,
        );

        let response = backend
            .translate_batch(request_with_items(vec![batch_item(0, "nested")]))
            .await
            .unwrap();

        assert_eq!(response.items[0].text, "嵌套文本");
    }

    #[tokio::test]
    async fn deeplx_extracts_translations_array_format() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "translations": [{ "text": "数组文本" }]
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let backend = DeepLXBackend::with_concurrency(
            client,
            format!("{}/translate", mock_server.uri()),
            DEFAULT_MAX_CONCURRENCY,
        );

        let response = backend
            .translate_batch(request_with_items(vec![batch_item(0, "array")]))
            .await
            .unwrap();

        assert_eq!(response.items[0].text, "数组文本");
    }
}
