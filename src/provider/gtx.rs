use async_trait::async_trait;
use reqwest::Client;

use crate::error::{MdTranslatorError, Result};
use crate::provider::concurrent::ConcurrentTranslationBackend;
use crate::provider::{TranslationBackend, provider_http_error};
use crate::types::{BatchItem, TranslateRequest, TranslateResponse, TranslateResponseItem};

#[derive(Debug, Clone)]
/// Google Translate web backend (no API key required).
pub struct GtxBackend {
    client: Client,
}

#[derive(Debug, Clone)]
pub struct GtxWorkUnit {
    item: BatchItem,
    source_lang: String,
    target_lang: String,
}

impl GtxBackend {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn translate_one(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let endpoint = format!(
            "https://translate.googleapis.com/translate_a/single?client=gtx&sl={source_lang}&tl={target_lang}&dt=t&q={}",
            urlencoding::encode(text)
        );

        let response = self.client.get(endpoint).send().await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(provider_http_error("gtx", status, &headers, body));
        }

        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        let arr = parsed.get(0).and_then(|v| v.as_array()).ok_or_else(|| {
            MdTranslatorError::InvalidResponse("GTX response missing translated array".to_string())
        })?;

        let mut out = String::new();
        for part in arr {
            if let Some(text_part) = part.get(0).and_then(|v| v.as_str()) {
                out.push_str(text_part);
            }
        }
        Ok(out)
    }
}

#[async_trait]
impl ConcurrentTranslationBackend for GtxBackend {
    type WorkUnit = GtxWorkUnit;
    type WorkOutput = TranslateResponseItem;

    fn split_work(&self, request: &TranslateRequest) -> Vec<Self::WorkUnit> {
        request
            .items
            .iter()
            .cloned()
            .map(|item| GtxWorkUnit {
                item,
                source_lang: request.source_lang.clone(),
                target_lang: request.target_lang.clone(),
            })
            .collect()
    }

    async fn execute_work_unit(&self, unit: Self::WorkUnit) -> Result<Self::WorkOutput> {
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
impl TranslationBackend for GtxBackend {
    fn name(&self) -> &'static str {
        "gtx"
    }

    fn cache_fingerprint(&self) -> String {
        "gtx.googleapis.com".to_string()
    }

    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
        self.execute_concurrently(&request, 1).await
    }
}
