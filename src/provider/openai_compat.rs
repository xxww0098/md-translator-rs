use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::error::{MdTranslatorError, Result};
use crate::provider::concurrent::ConcurrentTranslationBackend;
use crate::provider::xml::{
    ProviderXmlBatch, ProviderXmlBatchConfig, pack_batch_items_into_xml_batches,
    parse_provider_xml_response,
};
use crate::provider::{TranslationBackend, provider_http_error};
use crate::types::{BatchItem, TranslateRequest, TranslateResponse, TranslateResponseItem};

#[derive(Debug, Clone)]
/// OpenAI-compatible chat-completion backend with XML segment batching.
pub struct OpenAICompatBackend {
    client: Client,
    endpoint: String,
    api_key: String,
    model: String,
    temperature: f32,
    /// Maximum concurrent in-flight HTTP requests for XML batches.
    /// Derived from `RuntimeOptions::max_concurrency` so the provider
    /// respects the same ceiling as the engine's wave scheduler.
    max_concurrent_batches: usize,
}

#[derive(Debug, Clone)]
pub struct OpenAIWorkUnit {
    batch: ProviderXmlBatch,
    system_prompt: String,
    user_prompt: String,
}

impl OpenAICompatBackend {
    const DEFAULT_MAX_ITEMS_PER_BATCH: usize = 15;
    const DEFAULT_MAX_CONCURRENT_BATCHES: usize = 5;

    const CHAT_COMPLETIONS_PATH: &str = "/v1/chat/completions";

    pub fn new(
        client: Client,
        endpoint: String,
        api_key: String,
        model: String,
        temperature: f32,
    ) -> Self {
        Self::with_concurrency(
            client,
            endpoint,
            api_key,
            model,
            temperature,
            Self::DEFAULT_MAX_CONCURRENT_BATCHES,
        )
    }

    pub fn with_concurrency(
        client: Client,
        endpoint: String,
        api_key: String,
        model: String,
        temperature: f32,
        max_concurrent_batches: usize,
    ) -> Self {
        let endpoint = Self::normalize_endpoint(&endpoint);
        Self {
            client,
            endpoint,
            api_key,
            model,
            temperature,
            max_concurrent_batches,
        }
    }

    /// Normalizes the endpoint URL by appending `/v1/chat/completions` if not
    /// already present. This allows users to provide just a base URL
    /// (e.g. `http://localhost:8317`) in their configuration.
    fn normalize_endpoint(endpoint: &str) -> String {
        let trimmed = endpoint.trim_end_matches('/');
        if trimmed.ends_with("/chat/completions") {
            trimmed.to_string()
        } else if trimmed.ends_with("/v1") {
            format!("{trimmed}/chat/completions")
        } else {
            format!("{trimmed}{}", Self::CHAT_COMPLETIONS_PATH)
        }
    }

    fn default_system_prompt() -> String {
        "You are a professional translator. Translate each XML segment independently, preserve the XML tags exactly, and return only XML.".to_string()
    }

    fn default_user_prompt() -> String {
        "Translate the following XML batch from {source} to {target}. Preserve every <seg> tag and return only translated XML.

{text}".to_string()
    }

    fn batch_config() -> ProviderXmlBatchConfig {
        ProviderXmlBatchConfig {
            max_chars_per_batch: usize::MAX,
            max_items_per_batch: Self::DEFAULT_MAX_ITEMS_PER_BATCH,
        }
    }

    fn build_prompt(
        template: Option<String>,
        source_lang: &str,
        target_lang: &str,
        text: &str,
        default_template: fn() -> String,
    ) -> String {
        template
            .unwrap_or_else(default_template)
            .replace("{source}", source_lang)
            .replace("{target}", target_lang)
            .replace("{text}", text)
    }

    async fn send_chat_completion(
        &self,
        system_prompt: String,
        user_prompt: String,
    ) -> Result<String> {
        let response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.api_key)
            .json(&json!({
                "model": self.model,
                "temperature": self.temperature,
                "stream": false,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_prompt}
                ]
            }))
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(provider_http_error("openai-compat", status, &headers, body));
        }

        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        parsed
            .get("choices")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("message"))
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string)
            .ok_or_else(|| {
                MdTranslatorError::InvalidResponse(
                    "OpenAI compatible response missing choices[0].message.content".to_string(),
                )
            })
    }

    async fn translate_one_item(
        &self,
        request: &TranslateRequest,
        item: &BatchItem,
    ) -> Result<TranslateResponseItem> {
        let xml =
            pack_batch_items_into_xml_batches(std::slice::from_ref(item), &Self::batch_config())
                .into_iter()
                .next()
                .map(|batch| batch.xml)
                .ok_or_else(|| {
                    MdTranslatorError::InvalidResponse(
                        "OpenAI compatible single-item XML batch was unexpectedly empty"
                            .to_string(),
                    )
                })?;
        let system_prompt = Self::build_prompt(
            request.system_prompt.clone(),
            &request.source_lang,
            &request.target_lang,
            &xml,
            Self::default_system_prompt,
        );
        let user_prompt = Self::build_prompt(
            request.user_prompt.clone(),
            &request.source_lang,
            &request.target_lang,
            &xml,
            Self::default_user_prompt,
        );
        let content = self
            .send_chat_completion(system_prompt, user_prompt)
            .await?;
        let parsed = parse_provider_xml_response(&content, &[item.id]).map_err(|err| {
            MdTranslatorError::InvalidResponse(format!(
                "OpenAI compatible XML response parse failed: {err}"
            ))
        })?;

        let text = parsed.get(&item.id).cloned().ok_or_else(|| {
            MdTranslatorError::InvalidResponse(format!(
                "OpenAI compatible XML response missing translated segment for id={}",
                item.id
            ))
        })?;

        Ok(TranslateResponseItem { id: item.id, text })
    }
}

#[async_trait]
impl ConcurrentTranslationBackend for OpenAICompatBackend {
    type WorkUnit = OpenAIWorkUnit;
    type WorkOutput = (ProviderXmlBatch, String);

    fn split_work(&self, request: &TranslateRequest) -> Vec<Self::WorkUnit> {
        pack_batch_items_into_xml_batches(&request.items, &Self::batch_config())
            .into_iter()
            .map(|batch| OpenAIWorkUnit {
                system_prompt: Self::build_prompt(
                    request.system_prompt.clone(),
                    &request.source_lang,
                    &request.target_lang,
                    &batch.xml,
                    Self::default_system_prompt,
                ),
                user_prompt: Self::build_prompt(
                    request.user_prompt.clone(),
                    &request.source_lang,
                    &request.target_lang,
                    &batch.xml,
                    Self::default_user_prompt,
                ),
                batch,
            })
            .collect()
    }

    async fn execute_work_unit(&self, unit: Self::WorkUnit) -> Result<Self::WorkOutput> {
        let content = self
            .send_chat_completion(unit.system_prompt, unit.user_prompt)
            .await?;
        Ok((unit.batch, content))
    }

    async fn assemble_response(
        &self,
        request: &TranslateRequest,
        outputs: Vec<Self::WorkOutput>,
    ) -> Result<TranslateResponse> {
        let mut out = Vec::with_capacity(request.items.len());

        for (batch, content) in outputs {
            match parse_provider_xml_response(&content, &batch.ids) {
                Ok(parsed) => {
                    for id in batch.ids {
                        let text = parsed.get(&id).cloned().ok_or_else(|| {
                            MdTranslatorError::InvalidResponse(format!(
                                "OpenAI compatible XML response missing translated segment for id={id}"
                            ))
                        })?;
                        out.push(TranslateResponseItem { id, text });
                    }
                }
                Err(err) => {
                    if batch.ids.len() == 1 {
                        return Err(MdTranslatorError::InvalidResponse(format!(
                            "OpenAI compatible XML response parse failed: {err}"
                        )));
                    }

                    for item in request
                        .items
                        .iter()
                        .filter(|item| batch.ids.contains(&item.id))
                    {
                        out.push(self.translate_one_item(request, item).await?);
                    }
                }
            }
        }

        Ok(TranslateResponse { items: out })
    }
}

#[async_trait]
impl TranslationBackend for OpenAICompatBackend {
    fn name(&self) -> &'static str {
        "openai-compat"
    }

    fn cache_fingerprint(&self) -> String {
        format!(
            "endpoint={};model={};temperature={}",
            self.endpoint, self.model, self.temperature
        )
    }

    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
        if request.items.is_empty() {
            return Ok(TranslateResponse { items: Vec::new() });
        }

        self.execute_concurrently(&request, self.max_concurrent_batches)
            .await
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use serde_json::{Value, json};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_partial_json, body_string_contains, method, path},
    };

    use super::*;

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

    fn openai_response(content: &str) -> Value {
        json!({
            "choices": [{
                "message": {
                    "content": content
                }
            }]
        })
    }

    #[tokio::test]
    async fn openai_xml_batch_sends_single_request_for_15_items() {
        let mock_server = MockServer::start().await;
        let items: Vec<BatchItem> = (1..=15)
            .map(|id| batch_item(id, format!("text {id}")))
            .collect();

        let response_xml = (1..=15)
            .map(|id| format!(r#"<seg id="{id}">translated {id}</seg>"#))
            .collect::<Vec<_>>()
            .join("");

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_partial_json(json!({"model": "test-model"})))
            .and(body_string_contains("text 1"))
            .and(body_string_contains("text 15"))
            .respond_with(ResponseTemplate::new(200).set_body_json(openai_response(&response_xml)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let backend = OpenAICompatBackend::new(
            Client::new(),
            mock_server.uri(),
            "test-key".to_string(),
            "test-model".to_string(),
            0.0,
        );

        let response = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();

        assert_eq!(response.items.len(), 15);
        for (index, item) in response.items.iter().enumerate() {
            let id = index + 1;
            assert_eq!(item.id, id);
            assert_eq!(item.text, format!("translated {id}"));
        }
    }

    #[tokio::test]
    async fn openai_xml_batch_parse_failure_retries_individual_items() {
        let mock_server = MockServer::start().await;
        let items = vec![batch_item(1, "alpha"), batch_item(2, "beta")];

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("alpha"))
            .and(body_string_contains("beta"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(openai_response(r#"<seg id="1">only one</seg>"#)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("alpha"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(openai_response(r#"<seg id="1">translated alpha</seg>"#)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("beta"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(openai_response(r#"<seg id="2">translated beta</seg>"#)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let backend = OpenAICompatBackend::new(
            Client::new(),
            mock_server.uri(),
            "test-key".to_string(),
            "test-model".to_string(),
            0.0,
        );

        let response = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].id, 1);
        assert_eq!(response.items[0].text, "translated alpha");
        assert_eq!(response.items[1].id, 2);
        assert_eq!(response.items[1].text, "translated beta");
    }

    /// Verifies parse-failure fallback survives when OpenAI work runs through
    /// the shared scheduler path (`execute_concurrently`), not the provider-local
    /// `translate_batch_items` path.
    #[tokio::test]
    async fn openai_shared_scheduler_parse_failure_retries_individual_items() {
        let mock_server = MockServer::start().await;
        let items = vec![batch_item(1, "alpha"), batch_item(2, "beta")];

        // First batch request returns incomplete XML (only id=1, missing id=2)
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("alpha"))
            .and(body_string_contains("beta"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(openai_response(r#"<seg id="1">only one</seg>"#)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        // Fallback: per-item request for alpha
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("alpha"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(openai_response(r#"<seg id="1">translated alpha</seg>"#)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        // Fallback: per-item request for beta
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_string_contains("beta"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(openai_response(r#"<seg id="2">translated beta</seg>"#)),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let backend = OpenAICompatBackend::new(
            Client::new(),
            mock_server.uri(),
            "test-key".to_string(),
            "test-model".to_string(),
            0.0,
        );

        // Exercise the shared scheduler path directly
        let response = backend
            .execute_concurrently(&request_with_items(items), 5)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 2);
        assert_eq!(response.items[0].id, 1);
        assert_eq!(response.items[0].text, "translated alpha");
        assert_eq!(response.items[1].id, 2);
        assert_eq!(response.items[1].text, "translated beta");
    }

    /// Verifies normal XML batch path works through the shared scheduler
    /// (`execute_concurrently`), proving no regression in the happy path.
    #[tokio::test]
    async fn openai_shared_scheduler_sends_single_request_for_15_items() {
        let mock_server = MockServer::start().await;
        let items: Vec<BatchItem> = (1..=15)
            .map(|id| batch_item(id, format!("text {id}")))
            .collect();

        let response_xml = (1..=15)
            .map(|id| format!(r#"<seg id="{id}">translated {id}</seg>"#))
            .collect::<Vec<_>>()
            .join("");

        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .and(body_partial_json(json!({"model": "test-model"})))
            .and(body_string_contains("text 1"))
            .and(body_string_contains("text 15"))
            .respond_with(ResponseTemplate::new(200).set_body_json(openai_response(&response_xml)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let backend = OpenAICompatBackend::new(
            Client::new(),
            mock_server.uri(),
            "test-key".to_string(),
            "test-model".to_string(),
            0.0,
        );

        // Exercise the shared scheduler path directly
        let response = backend
            .execute_concurrently(&request_with_items(items), 5)
            .await
            .unwrap();

        assert_eq!(response.items.len(), 15);
        for (index, item) in response.items.iter().enumerate() {
            let id = index + 1;
            assert_eq!(item.id, id);
            assert_eq!(item.text, format!("translated {id}"));
        }
    }

    #[test]
    fn normalize_endpoint_appends_path_to_base_url() {
        assert_eq!(
            OpenAICompatBackend::normalize_endpoint("http://localhost:8317"),
            "http://localhost:8317/v1/chat/completions"
        );
    }

    #[test]
    fn normalize_endpoint_appends_path_to_base_url_with_trailing_slash() {
        assert_eq!(
            OpenAICompatBackend::normalize_endpoint("http://localhost:8317/"),
            "http://localhost:8317/v1/chat/completions"
        );
    }

    #[test]
    fn normalize_endpoint_keeps_full_path() {
        assert_eq!(
            OpenAICompatBackend::normalize_endpoint("http://localhost:8317/v1/chat/completions"),
            "http://localhost:8317/v1/chat/completions"
        );
    }

    #[test]
    fn normalize_endpoint_completes_v1_path() {
        assert_eq!(
            OpenAICompatBackend::normalize_endpoint("http://localhost:8317/v1"),
            "http://localhost:8317/v1/chat/completions"
        );
    }

    #[test]
    fn normalize_endpoint_keeps_custom_chat_completions_path() {
        assert_eq!(
            OpenAICompatBackend::normalize_endpoint(
                "http://proxy.example.com/custom/chat/completions"
            ),
            "http://proxy.example.com/custom/chat/completions"
        );
    }
}
