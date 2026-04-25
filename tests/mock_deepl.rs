mod common;

use std::time::Duration;

use md_translator_rs::{DeepLBackend, MdTranslatorError, TranslationBackend};
use reqwest::Client;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string_contains, method, path},
};

use common::{batch_item, request_with_items};

#[tokio::test]
async fn mock_deepl_success() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/translate"))
        .and(body_string_contains("Hello"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"translations": [{"text": "\u4f60\u597d"}]}"#),
        )
        .mount(&mock_server)
        .await;

    let backend = DeepLBackend::new(
        Client::new(),
        format!("{}/translate", mock_server.uri()),
        "test-key".to_string(),
    );

    let response = backend
        .translate_batch(request_with_items(items))
        .await
        .unwrap();
    assert_eq!(response.items.len(), 1);
    assert_eq!(response.items[0].text, "你好");
}

#[tokio::test]
async fn mock_deepl_timeout() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/translate"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .mount(&mock_server)
        .await;

    let client = Client::builder()
        .timeout(Duration::from_millis(50))
        .build()
        .unwrap();
    let backend = DeepLBackend::new(
        client,
        format!("{}/translate", mock_server.uri()),
        "test-key".to_string(),
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
async fn mock_deepl_429() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/translate"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Too Many Requests"))
        .mount(&mock_server)
        .await;

    let backend = DeepLBackend::new(
        Client::new(),
        format!("{}/translate", mock_server.uri()),
        "test-key".to_string(),
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
            assert_eq!(provider, "deepl");
            assert_eq!(status_code, 429);
            assert_eq!(retry_after_ms, None);
            assert!(message.contains("Too Many Requests"));
        }
        other => panic!("expected ProviderHttp error, got {other:?}"),
    }
}

#[tokio::test]
async fn mock_deepl_500() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/translate"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    let backend = DeepLBackend::new(
        Client::new(),
        format!("{}/translate", mock_server.uri()),
        "test-key".to_string(),
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
            assert_eq!(provider, "deepl");
            assert_eq!(status_code, 500);
            assert_eq!(retry_after_ms, None);
            assert!(message.contains("Internal Server Error"));
        }
        other => panic!("expected ProviderHttp error, got {other:?}"),
    }
}

#[tokio::test]
async fn mock_deepl_malformed_response() {
    let mock_server = MockServer::start().await;
    let items = vec![batch_item(0, "Hello")];

    Mock::given(method("POST"))
        .and(path("/translate"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&mock_server)
        .await;

    let backend = DeepLBackend::new(
        Client::new(),
        format!("{}/translate", mock_server.uri()),
        "test-key".to_string(),
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
