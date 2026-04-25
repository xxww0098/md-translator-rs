use thiserror::Error;

use crate::config::ConfigError;

/// Errors produced by the translation pipeline.
#[derive(Debug, Error)]
pub enum MdTranslatorError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("utf8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("config error: {0}")]
    Config(#[from] ConfigError),

    #[error("provider error: {0}")]
    Provider(String),

    #[error(
        "provider http error ({provider}, status {status_code}{retry_after_suffix}): {message}",
        retry_after_suffix = retry_after_suffix(*retry_after_ms)
    )]
    ProviderHttp {
        provider: &'static str,
        status_code: u16,
        retry_after_ms: Option<u64>,
        message: String,
    },

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("batch mismatch: expected {expected}, got {actual}")]
    BatchMismatch { expected: usize, actual: usize },

    #[error("task join error: {0}")]
    Join(String),
}

pub type Result<T> = std::result::Result<T, MdTranslatorError>;

fn retry_after_suffix(retry_after_ms: Option<u64>) -> String {
    retry_after_ms
        .map(|ms| format!(", retry-after={}ms", ms))
        .unwrap_or_default()
}
