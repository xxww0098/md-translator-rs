use std::any::Any;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, RETRY_AFTER};

use crate::error::{MdTranslatorError, Result};
use crate::types::{TranslateRequest, TranslateResponse};

pub(crate) const MAX_RETRY_AFTER_MS: u64 = 300_000;

pub mod concurrent;
pub mod deepl;
pub mod deeplx;
pub mod gtx;
pub mod openai_compat;
pub mod xml;

pub use deepl::DeepLBackend;
pub use deeplx::DeepLXBackend;
pub use gtx::GtxBackend;
pub use openai_compat::OpenAICompatBackend;

/// Translation provider interface.
///
/// Implement this trait to add a new translation backend.
/// Built-in implementations: [`GtxBackend`], [`DeepLXBackend`],
/// [`OpenAICompatBackend`], [`DeepLBackend`].
/// Provider-internal work-unit scheduling stays under [`self::concurrent`] so
/// the engine remains responsible only for document/batch orchestration.
#[async_trait]
pub trait TranslationBackend: Send + Sync + Any {
    /// Human-readable provider name (e.g. `"gtx"`, `"deepl"`).
    fn name(&self) -> &'static str;
    /// Stable fingerprint for cache key differentiation between provider configurations.
    fn cache_fingerprint(&self) -> String;
    /// Translate a batch of items in a single provider request.
    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse>;
}

pub(crate) fn provider_http_error(
    provider: &'static str,
    status: StatusCode,
    headers: &HeaderMap,
    body: String,
) -> MdTranslatorError {
    MdTranslatorError::ProviderHttp {
        provider,
        status_code: status.as_u16(),
        retry_after_ms: retry_after_from_headers(headers),
        message: body,
    }
}

fn retry_after_from_headers(headers: &HeaderMap) -> Option<u64> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();

    if let Ok(seconds) = value.parse::<u64>() {
        return Some(cap_retry_after_ms(seconds.saturating_mul(1000)));
    }

    let retry_at = httpdate::parse_http_date(value).ok()?;
    let now = SystemTime::now();
    let duration = retry_at.duration_since(now).ok()?;
    Some(cap_retry_after_ms(duration_to_millis(duration)))
}

fn duration_to_millis(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

fn cap_retry_after_ms(retry_after_ms: u64) -> u64 {
    retry_after_ms.min(MAX_RETRY_AFTER_MS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

    #[test]
    fn numeric_retry_after_is_capped() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("86400"));

        assert_eq!(retry_after_from_headers(&headers), Some(MAX_RETRY_AFTER_MS));
    }
}
