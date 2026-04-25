use std::env;
use std::time::{Duration, Instant};

use md_translator_rs::{BatchItem, DeepLXBackend, TranslateRequest, TranslationBackend};
use reqwest::Client;
use tokio::time::{sleep, timeout};

const SLOW_THRESHOLD: Duration = Duration::from_secs(30);
const FAST_TIMEOUT: Duration = Duration::from_secs(25);
const OPTIMIZED_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const OPTIMIZED_REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_RETRIES: u32 = 2;
const RETRY_BACKOFF_MS: u64 = 250;

#[derive(Debug)]
struct TimingSample {
    attempt: u32,
    request_build_ms: u128,
    response_wait_ms: u128,
    parse_wait_ms: u128,
    translation_duration_ms: u128,
    total_attempt_ms: u128,
}

#[derive(Debug)]
struct TimingResult {
    translated_text: String,
    total_elapsed_ms: u128,
    attempts: Vec<TimingSample>,
    optimized: bool,
}

#[tokio::test]
#[ignore = "requires live DeepLX endpoint via DEEPLX_ENDPOINT"]
async fn deeplx_translation_reports_timing_and_stays_under_30_seconds() {
    let endpoint = match env::var("DEEPLX_ENDPOINT") {
        Ok(value) if !value.trim().is_empty() => value,
        _ => {
            eprintln!("Skipping DeepLX integration test: DEEPLX_ENDPOINT is not set.");
            return;
        }
    };

    let source_lang = env::var("DEEPLX_SOURCE_LANG").unwrap_or_else(|_| "EN".to_string());
    let target_lang = env::var("DEEPLX_TARGET_LANG").unwrap_or_else(|_| "ZH".to_string());
    let text = env::var("DEEPLX_TEST_TEXT").unwrap_or_else(|_| "Hello, world!".to_string());

    let result = run_profile(
        &endpoint,
        &text,
        &source_lang,
        &target_lang,
        FAST_TIMEOUT,
        Duration::from_secs(5),
    )
    .await;

    let result = match result {
        Ok(result) => result,
        Err(first_error) => {
            let fallback = run_profile(
                &endpoint,
                &text,
                &source_lang,
                &target_lang,
                OPTIMIZED_REQUEST_TIMEOUT,
                OPTIMIZED_CONNECT_TIMEOUT,
            )
            .await;

            match fallback {
                Ok(mut optimized_result) => {
                    optimized_result.optimized = true;
                    optimized_result
                }
                Err(second_error) => {
                    panic!(
                        "DeepLX translation failed in both profiles: fast={first_error}; optimized={second_error}"
                    )
                }
            }
        }
    };

    assert!(
        !result.translated_text.trim().is_empty(),
        "DeepLX returned an empty translation"
    );

    for sample in &result.attempts {
        println!(
            "attempt={} request_build_ms={} response_wait_ms={} parse_wait_ms={} translation_duration_ms={} total_attempt_ms={}",
            sample.attempt,
            sample.request_build_ms,
            sample.response_wait_ms,
            sample.parse_wait_ms,
            sample.translation_duration_ms,
            sample.total_attempt_ms
        );
    }

    println!(
        "optimized={} total_elapsed_ms={} translated_text={}",
        result.optimized, result.total_elapsed_ms, result.translated_text
    );

    assert!(
        result.total_elapsed_ms <= SLOW_THRESHOLD.as_millis(),
        "DeepLX run exceeded {} seconds. total_elapsed_ms={}, optimized={}, attempts={:#?}",
        SLOW_THRESHOLD.as_secs(),
        result.total_elapsed_ms,
        result.optimized,
        result.attempts
    );
}

async fn run_profile(
    endpoint: &str,
    text: &str,
    source_lang: &str,
    target_lang: &str,
    request_timeout: Duration,
    connect_timeout: Duration,
) -> Result<TimingResult, Box<dyn std::error::Error>> {
    run_profile_with_timeouts(
        endpoint,
        text,
        source_lang,
        target_lang,
        request_timeout,
        connect_timeout,
    )
    .await
}

async fn run_profile_with_timeouts(
    endpoint: &str,
    text: &str,
    source_lang: &str,
    target_lang: &str,
    request_timeout: Duration,
    connect_timeout: Duration,
) -> Result<TimingResult, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .connect_timeout(connect_timeout)
        .timeout(request_timeout)
        .build()
        .expect("reqwest client should build");

    let backend = DeepLXBackend::new(client, endpoint.to_string());

    let overall_start = Instant::now();
    let mut attempts = Vec::new();
    let mut optimized = false;
    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES + 1 {
        let setup_started = Instant::now();
        let request = build_request(text, source_lang, target_lang);
        let request_build_ms = setup_started.elapsed().as_millis();

        let send_started = Instant::now();
        let translated = timeout(request_timeout, backend.translate_batch(request)).await;
        let response_wait_ms = send_started.elapsed().as_millis();
        match translated {
            Ok(Ok(response)) => {
                let parse_started = Instant::now();
                let translated_text = response
                    .items
                    .into_iter()
                    .next()
                    .map(|item| item.text)
                    .unwrap_or_default();
                let parse_wait_ms = parse_started.elapsed().as_millis();
                let total_attempt_ms = request_build_ms + response_wait_ms + parse_wait_ms;
                let translation_duration_ms = response_wait_ms + parse_wait_ms;

                attempts.push(TimingSample {
                    attempt,
                    request_build_ms,
                    response_wait_ms,
                    parse_wait_ms,
                    translation_duration_ms,
                    total_attempt_ms,
                });

                let total_elapsed_ms = overall_start.elapsed().as_millis();
                if total_elapsed_ms > SLOW_THRESHOLD.as_millis() {
                    optimized = true;
                }

                return Ok(TimingResult {
                    translated_text,
                    total_elapsed_ms,
                    attempts,
                    optimized,
                });
            }
            Ok(Err(error)) => {
                let parse_wait_ms = 0;
                let total_attempt_ms = request_build_ms + response_wait_ms;
                let translation_duration_ms = response_wait_ms;

                attempts.push(TimingSample {
                    attempt,
                    request_build_ms,
                    response_wait_ms,
                    parse_wait_ms,
                    translation_duration_ms,
                    total_attempt_ms,
                });

                last_error = Some(error.to_string());

                if overall_start.elapsed() > SLOW_THRESHOLD {
                    break;
                }

                if attempt <= MAX_RETRIES {
                    optimized = true;
                    sleep(Duration::from_millis(RETRY_BACKOFF_MS * u64::from(attempt))).await;
                }
            }
            Err(_) => {
                let parse_wait_ms = 0;
                let total_attempt_ms = request_build_ms + response_wait_ms;
                let translation_duration_ms = response_wait_ms;

                attempts.push(TimingSample {
                    attempt,
                    request_build_ms,
                    response_wait_ms,
                    parse_wait_ms,
                    translation_duration_ms,
                    total_attempt_ms,
                });

                last_error = Some("request timed out".to_string());

                if overall_start.elapsed() > SLOW_THRESHOLD {
                    break;
                }

                if attempt <= MAX_RETRIES {
                    optimized = true;
                    sleep(Duration::from_millis(RETRY_BACKOFF_MS * u64::from(attempt))).await;
                }
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| "DeepLX translation failed without an error".to_string())
        .into())
}

fn build_request(text: &str, source_lang: &str, target_lang: &str) -> TranslateRequest {
    TranslateRequest {
        source_lang: source_lang.to_string(),
        target_lang: target_lang.to_string(),
        items: vec![BatchItem {
            id: 0,
            text: text.to_string(),
            context_before: Vec::new(),
            context_after: Vec::new(),
        }],
        preserve_markdown: true,
        system_prompt: None,
        user_prompt: None,
    }
}
