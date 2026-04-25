mod common;

use std::time::Duration;

use md_translator_rs::{MdTranslatorError, OpenAICompatBackend, TranslationBackend};
use reqwest::Client;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, method, path},
};

use common::{batch_item, request_with_items};

#[tokio::test]
async fn mock_openai_success() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_string_contains("Hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"choices": [{"message": {"content": "<seg id=\"0\">\u4f60\u597d</seg>"}}]}"#,
        ))
        .mount(&mock_server)
        .await;

    let backend = OpenAICompatBackend::with_concurrency(
        Client::new(),
        mock_server.uri(),
        "test-key".to_string(),
        "test-model".to_string(),
        0.0,
        5,
    );

    let response = backend
        .translate_batch(request_with_items(items))
        .await
        .unwrap();
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].text, "你好");
}

#[tokio::test]
async fn mock_openai_timeout() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .timeout(Duration::from_millis(50))
        .build()
        .unwrap();
    let backend = OpenAICompatBackend::with_concurrency(
        client,
        mock_server.uri(),
        "test-key".to_string(),
        "test-model".to_string(),
        0.0,
        5,
    );

    let err = backend
        .translate_batch(request_with_items(items))
        .await
        .unwrap_err();
    assert!(
        matches!(err, MdTranslatorError::Http(_)),
        "expected Http timeout error, got {err:?}"
    );
}

#[tokio::test]
async fn mock_openai_429() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Too Many Requests"))
        .mount(&mock_server)
        .await;

    let backend = OpenAICompatBackend::with_concurrency(
        Client::new(),
        mock_server.uri(),
        "test-key".to_string(),
        "test-model".to_string(),
        0.0,
        5,
    );

    let err = backend
        .translate_batch(request_with_items(items))
        .await
        .unwrap_err();
    match err {
        MdTranslatorError::ProviderHttp {
            provider,
            status_code,
            retry_after_ms,
            message,
        } => {
            assert_eq!(provider, "openai-compat");
            assert_eq!(status_code, 429);
            assert_eq!(retry_after_ms, None);
            assert!(message.contains("Too Many Requests"));
        }
        other => panic!("expected ProviderHttp error, got {other:?}"),
    }
}

#[tokio::test]
async fn mock_openai_500() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let backend = OpenAICompatBackend::with_concurrency(
        Client::new(),
        mock_server.uri(),
        "test-key".to_string(),
        "test-model".to_string(),
        0.0,
        5,
    );

    let err = backend
        .translate_batch(request_with_items(items))
        .await
        .unwrap_err();
    match err {
        MdTranslatorError::ProviderHttp {
            provider,
            status_code,
            retry_after_ms,
            message,
        } => {
            assert_eq!(provider, "openai-compat");
            assert_eq!(status_code, 500);
            assert_eq!(retry_after_ms, None);
            assert!(message.contains("Internal Server Error"));
        }
        other => panic!("expected ProviderHttp error, got {other:?}"),
    }
}

#[tokio::test]
async fn mock_openai_malformed_response() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&mock_server)
        .await;

    let backend = OpenAICompatBackend::with_concurrency(
        Client::new(),
        mock_server.uri(),
        "test-key".to_string(),
        "test-model".to_string(),
        0.0,
        5,
    );

    let err = backend
        .translate_batch(request_with_items(items))
        .await
        .unwrap_err();
    assert!(
        matches!(err, MdTranslatorError::Json(_)),
        "expected Json error, got {err:?}"
    );
}
