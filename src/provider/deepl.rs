use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::error::{MdTranslatorError, Result};
use crate::provider::concurrent::ConcurrentTranslationBackend;
use crate::provider::{TranslationBackend, provider_http_error};
use crate::types::{BatchItem, TranslateRequest, TranslateResponse, TranslateResponseItem};

#[derive(Debug, Clone)]
/// DeepL official API backend with native batching (up to 50 texts per request).
pub struct DeepLBackend {
    client: Client,
    endpoint: String,
    api_key: String,
}

#[derive(Debug, Clone)]
pub struct DeepLChunkWorkUnit {
    items: Vec<BatchItem>,
    source_lang: String,
    target_lang: String,
    preserve_markdown: bool,
}

impl DeepLBackend {
    const MAX_TEXTS_PER_REQUEST: usize = 50;
    const MAX_BYTES_PER_REQUEST: usize = 130_000;

    pub fn new(client: Client, endpoint: String, api_key: String) -> Self {
        Self {
            client,
            endpoint,
            api_key,
        }
    }

    async fn translate_chunk(
        &self,
        items: &[BatchItem],
        source_lang: &str,
        target_lang: &str,
        preserve_markdown: bool,
    ) -> Result<Vec<TranslateResponseItem>> {
        let texts: Vec<&str> = items.iter().map(|i| i.text.as_str()).collect();

        let source_lang = if source_lang == "auto" {
            serde_json::Value::Null
        } else {
            serde_json::Value::String(source_lang.to_uppercase())
        };

        let response = self
            .client
            .post(&self.endpoint)
            .json(&json!({
                "text": texts,
                "source_lang": source_lang,
                "target_lang": target_lang.to_uppercase(),
                "tag_handling": if preserve_markdown { "html" } else { "" },
                "auth_key": self.api_key,
            }))
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(provider_http_error("deepl", status, &headers, body));
        }

        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        let translations = parsed
            .get("translations")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                MdTranslatorError::InvalidResponse(
                    "DeepL response missing translations array".to_string(),
                )
            })?;

        if translations.len() != items.len() {
            return Err(MdTranslatorError::InvalidResponse(format!(
                "DeepL returned {} translations for {} texts",
                translations.len(),
                items.len()
            )));
        }

        let mut out = Vec::with_capacity(items.len());
        for (item, translation) in items.iter().zip(translations.iter()) {
            let text = translation
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    MdTranslatorError::InvalidResponse(
                        "DeepL response missing translation text".to_string(),
                    )
                })?;
            out.push(TranslateResponseItem {
                id: item.id,
                text: text.to_string(),
            });
        }

        Ok(out)
    }
}

#[async_trait]
impl ConcurrentTranslationBackend for DeepLBackend {
    type WorkUnit = DeepLChunkWorkUnit;
    type WorkOutput = Vec<TranslateResponseItem>;

    fn split_work(&self, request: &TranslateRequest) -> Vec<Self::WorkUnit> {
        let mut chunks: Vec<Self::WorkUnit> = Vec::new();
        let mut current_chunk: Vec<BatchItem> = Vec::new();
        let mut current_bytes: usize = 0;

        for item in &request.items {
            let item_bytes = item.text.len();
            if !current_chunk.is_empty()
                && (current_chunk.len() >= Self::MAX_TEXTS_PER_REQUEST
                    || current_bytes + item_bytes > Self::MAX_BYTES_PER_REQUEST)
            {
                chunks.push(DeepLChunkWorkUnit {
                    items: std::mem::take(&mut current_chunk),
                    source_lang: request.source_lang.clone(),
                    target_lang: request.target_lang.clone(),
                    preserve_markdown: request.preserve_markdown,
                });
                current_bytes = 0;
            }
            current_bytes += item_bytes;
            current_chunk.push(item.clone());
        }

        if !current_chunk.is_empty() {
            chunks.push(DeepLChunkWorkUnit {
                items: current_chunk,
                source_lang: request.source_lang.clone(),
                target_lang: request.target_lang.clone(),
                preserve_markdown: request.preserve_markdown,
            });
        }

        chunks
    }

    async fn execute_work_unit(&self, unit: Self::WorkUnit) -> Result<Self::WorkOutput> {
        self.translate_chunk(
            &unit.items,
            &unit.source_lang,
            &unit.target_lang,
            unit.preserve_markdown,
        )
        .await
    }

    async fn assemble_response(
        &self,
        request: &TranslateRequest,
        outputs: Vec<Self::WorkOutput>,
    ) -> Result<TranslateResponse> {
        let mut out = Vec::with_capacity(request.items.len());
        for mut chunk in outputs {
            out.append(&mut chunk);
        }
        Ok(TranslateResponse { items: out })
    }
}

#[async_trait]
impl TranslationBackend for DeepLBackend {
    fn name(&self) -> &'static str {
        "deepl"
    }

    fn cache_fingerprint(&self) -> String {
        format!("endpoint={}", self.endpoint)
    }

    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
        if request.items.is_empty() {
            return Ok(TranslateResponse { items: Vec::new() });
        }

        self.execute_concurrently(&request, 1).await
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_partial_json, method, path},
    };

    use super::*;

    #[tokio::test]
    async fn deepl_batch_50_sends_single_request() {
        let mock_server = MockServer::start().await;

        let items: Vec<BatchItem> = (0..50)
            .map(|i| BatchItem {
                id: i,
                text: format!("text {}", i),
                context_before: vec![],
                context_after: vec![],
            })
            .collect();

        let expected_texts: Vec<String> = items.iter().map(|i| i.text.clone()).collect();
        let translations: Vec<serde_json::Value> = items
            .iter()
            .map(|i| json!({"text": format!("translated {}", i.id)}))
            .collect();

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_partial_json(json!({"text": expected_texts})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "translations": translations
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let backend = DeepLBackend::new(
            client,
            format!("{}/translate", mock_server.uri()),
            "test-key".to_string(),
        );

        let request = TranslateRequest {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            items,
            preserve_markdown: true,
            system_prompt: None,
            user_prompt: None,
        };

        let response = backend.translate_batch(request).await.unwrap();

        assert_eq!(response.items.len(), 50);
        for (i, item) in response.items.iter().enumerate() {
            assert_eq!(item.id, i);
            assert_eq!(item.text, format!("translated {}", i));
        }
    }

    #[tokio::test]
    async fn deepl_batch_51_sends_two_requests() {
        let mock_server = MockServer::start().await;

        let items: Vec<BatchItem> = (0..51)
            .map(|i| BatchItem {
                id: i,
                text: format!("text {}", i),
                context_before: vec![],
                context_after: vec![],
            })
            .collect();

        let first_50_texts: Vec<String> = items[..50].iter().map(|i| i.text.clone()).collect();
        let first_50_translations: Vec<serde_json::Value> = items[..50]
            .iter()
            .map(|i| json!({"text": format!("translated {}", i.id)}))
            .collect();

        let last_1_texts: Vec<String> = items[50..].iter().map(|i| i.text.clone()).collect();
        let last_1_translations: Vec<serde_json::Value> = items[50..]
            .iter()
            .map(|i| json!({"text": format!("translated {}", i.id)}))
            .collect();

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_partial_json(json!({"text": first_50_texts})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "translations": first_50_translations
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_partial_json(json!({"text": last_1_texts})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "translations": last_1_translations
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let backend = DeepLBackend::new(
            client,
            format!("{}/translate", mock_server.uri()),
            "test-key".to_string(),
        );

        let request = TranslateRequest {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            items,
            preserve_markdown: true,
            system_prompt: None,
            user_prompt: None,
        };

        let response = backend.translate_batch(request).await.unwrap();

        assert_eq!(response.items.len(), 51);
        for (i, item) in response.items.iter().enumerate() {
            assert_eq!(item.id, i);
            assert_eq!(item.text, format!("translated {}", i));
        }
    }

    #[tokio::test]
    async fn deepl_batch_respects_byte_limit() {
        let mock_server = MockServer::start().await;

        let long_text = "x".repeat(3000);
        let items: Vec<BatchItem> = (0..45)
            .map(|i| BatchItem {
                id: i,
                text: long_text.clone(),
                context_before: vec![],
                context_after: vec![],
            })
            .collect();

        let chunk1_size = 43;
        let first_texts: Vec<String> = items[..chunk1_size]
            .iter()
            .map(|i| i.text.clone())
            .collect();
        let first_translations: Vec<serde_json::Value> = items[..chunk1_size]
            .iter()
            .map(|i| json!({"text": format!("translated {}", i.id)}))
            .collect();

        let last_texts: Vec<String> = items[chunk1_size..]
            .iter()
            .map(|i| i.text.clone())
            .collect();
        let last_translations: Vec<serde_json::Value> = items[chunk1_size..]
            .iter()
            .map(|i| json!({"text": format!("translated {}", i.id)}))
            .collect();

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_partial_json(json!({"text": first_texts})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "translations": first_translations
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/translate"))
            .and(body_partial_json(json!({"text": last_texts})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "translations": last_translations
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let backend = DeepLBackend::new(
            client,
            format!("{}/translate", mock_server.uri()),
            "test-key".to_string(),
        );

        let request = TranslateRequest {
            source_lang: "en".to_string(),
            target_lang: "zh".to_string(),
            items,
            preserve_markdown: true,
            system_prompt: None,
            user_prompt: None,
        };

        let response = backend.translate_batch(request).await.unwrap();

        assert_eq!(response.items.len(), 45);
        for (i, item) in response.items.iter().enumerate() {
            assert_eq!(item.id, i);
            assert_eq!(item.text, format!("translated {}", i));
        }
    }
}
