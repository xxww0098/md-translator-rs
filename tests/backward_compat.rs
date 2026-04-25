use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use async_trait::async_trait;
use clap::{CommandFactory, Parser};

use md_translator_rs::{
    Cli, MdTranslator, TranslateOptions, TranslateRequest, TranslateResponse,
    TranslateResponseItem, TranslationBackend, TranslationReporter, TwoTierCache, Verbosity,
};

struct IdentityBackend {
    calls: AtomicUsize,
}

struct RecordingBackend {
    requests: std::sync::Mutex<Vec<Vec<String>>>,
}

#[async_trait]
impl TranslationBackend for IdentityBackend {
    fn name(&self) -> &'static str {
        "identity"
    }

    fn cache_fingerprint(&self) -> String {
        "identity".to_string()
    }

    async fn translate_batch(
        &self,
        request: TranslateRequest,
    ) -> md_translator_rs::Result<TranslateResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(TranslateResponse {
            items: request
                .items
                .into_iter()
                .map(|item| TranslateResponseItem {
                    id: item.id,
                    text: format!("tr {}", item.text),
                })
                .collect(),
        })
    }
}

#[async_trait]
impl TranslationBackend for RecordingBackend {
    fn name(&self) -> &'static str {
        "recording"
    }

    fn cache_fingerprint(&self) -> String {
        "recording".to_string()
    }

    async fn translate_batch(
        &self,
        request: TranslateRequest,
    ) -> md_translator_rs::Result<TranslateResponse> {
        self.requests
            .lock()
            .unwrap()
            .push(request.items.iter().map(|item| item.text.clone()).collect());

        Ok(TranslateResponse {
            items: request
                .items
                .into_iter()
                .map(|item| TranslateResponseItem {
                    id: item.id,
                    text: format!("tr {}", item.text),
                })
                .collect(),
        })
    }
}

#[test]
fn cli_help_contains_expected_flags() {
    let mut cmd = <Cli as CommandFactory>::command();
    let help = cmd.render_help().to_string();
    assert!(help.contains("--input"));
    assert!(help.contains("--target"));
    assert!(help.contains("--provider-kind"));
    assert!(help.contains("--provider"));
    assert!(help.contains("--config"));
    assert!(help.contains("--quiet"));
    assert!(help.contains("--verbose"));
    assert!(help.contains("--batch-size"));
    assert!(help.contains("--no-cache"));
}

#[test]
fn readme_example_1_gtx_parses() {
    let args = vec![
        "mdxlate",
        "--input",
        "/tmp/test.md",
        "--target",
        "zh",
        "--provider-kind",
        "ordinary",
        "--provider",
        "gtx",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "README example 1 should parse: {:?}",
        cli.err()
    );
}

#[test]
fn readme_example_2_deeplx_explicit_parses() {
    let args = vec![
        "mdxlate",
        "--input",
        "/tmp/test.md",
        "--target",
        "zh",
        "--provider-kind",
        "ordinary",
        "--provider",
        "deep-lx",
        "--deeplx-endpoint",
        "http://127.0.0.1:1188/translate",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "README example 2 (explicit) should parse: {:?}",
        cli.err()
    );
}

#[test]
fn readme_example_3_openai_explicit_parses() {
    let args = vec![
        "mdxlate",
        "--input",
        "/tmp/test.md",
        "--target",
        "zh",
        "--provider-kind",
        "ai",
        "--provider",
        "openai-compat",
        "--api-key",
        "test-key",
        "--openai-endpoint",
        "https://api.openai.com/v1/chat/completions",
        "--model",
        "gpt-4o-mini",
        "--temperature",
        "0.2",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "README example 3 (explicit) should parse: {:?}",
        cli.err()
    );
}

#[test]
fn readme_example_4_deepl_parses() {
    let args = vec![
        "mdxlate",
        "--input",
        "/tmp/test.md",
        "--target",
        "zh",
        "--provider-kind",
        "ai",
        "--provider",
        "deep-l",
        "--api-key",
        "test-key",
        "--deepl-endpoint",
        "https://api-free.deepl.com/v2/translate",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "README example 4 should parse: {:?}",
        cli.err()
    );
}

#[test]
fn readme_example_5_explicit_output_parses() {
    let args = vec![
        "mdxlate",
        "--input",
        "/tmp/test.md",
        "--output",
        "/tmp/out.md",
        "--target",
        "zh",
        "--provider-kind",
        "ordinary",
        "--provider",
        "gtx",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "README example 5 should parse: {:?}",
        cli.err()
    );
}

#[test]
fn readme_example_6_no_cache_parses() {
    let args = vec![
        "mdxlate",
        "--input",
        "/tmp/test.md",
        "--target",
        "zh",
        "--provider-kind",
        "ordinary",
        "--provider",
        "gtx",
        "--no-cache",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "README example 6 should parse: {:?}",
        cli.err()
    );
}

#[test]
fn readme_common_options_parses() {
    let args = vec![
        "mdxlate",
        "--input",
        "/tmp/test.md",
        "--target",
        "zh",
        "--provider-kind",
        "ordinary",
        "--provider",
        "gtx",
        "--format",
        "markdown",
        "--source",
        "en",
        "--translate-frontmatter",
        "--translate-multiline-code",
        "--translate-latex",
        "--no-translate-link-text",
        "--batch-size",
        "20",
        "--max-chars-per-batch",
        "5000",
        "--context-window",
        "50",
        "--concurrency",
        "6",
        "--retries",
        "3",
        "--retry-backoff-ms",
        "300",
        "--timeout-secs",
        "60",
        "--system-prompt",
        "You are a professional translator.",
        "--user-prompt",
        "Translate from {source} to {target}:\n\n{text}",
    ];
    let cli = Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "Common options example should parse: {:?}",
        cli.err()
    );
}

#[tokio::test]
async fn readme_library_example_runs_with_mock() {
    let _client = reqwest::Client::new();
    let backend = Arc::new(IdentityBackend {
        calls: AtomicUsize::new(0),
    });
    let cache_dir =
        std::env::temp_dir().join(format!("md-translator-bench-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(backend.clone(), cache);

    let options = TranslateOptions::default();
    let translated = translator
        .translate_markdown("# Hello world\n\nThis is a test.", &options)
        .await
        .unwrap();

    assert!(translated.contains("tr "));
    assert!(translated.contains("#"));
}

#[tokio::test]
async fn markdown_runtime_uses_ast_node_batches() {
    let backend = Arc::new(RecordingBackend {
        requests: std::sync::Mutex::new(Vec::new()),
    });
    let cache_dir =
        std::env::temp_dir().join(format!("md-translator-runtime-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let translator = MdTranslator::new(backend.clone(), cache);

    let options = TranslateOptions::default();
    let input = "# Heading\n\nParagraph with [link](https://example.com).";
    let translated = translator
        .translate_markdown(input, &options)
        .await
        .unwrap();

    let requests = backend.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0], vec!["Heading", "Paragraph with .", "link"]);
    assert!(translated.contains("# tr Heading"));
    assert!(translated.contains("[tr link](https://example.com)"));
}

#[tokio::test]
async fn runtime_reporter_tracks_requests_cache_hits_and_progress() {
    let backend = Arc::new(IdentityBackend {
        calls: AtomicUsize::new(0),
    });
    let cache_dir =
        std::env::temp_dir().join(format!("md-translator-reporter-{}", std::process::id()));
    let cache = Arc::new(TwoTierCache::new(cache_dir).unwrap());
    let reporter = TranslationReporter::new(Verbosity::Normal, 1);
    let translator = MdTranslator::new(backend.clone(), cache).with_reporter(reporter.clone());

    let options = TranslateOptions::default();
    let input = "# Heading\n\nParagraph.";

    let first = translator
        .translate_markdown_with_report(input, &options)
        .await
        .unwrap();

    assert_eq!(first.summary.total_lines, 2);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
    assert_eq!(reporter.state().requests_sent(), 1);
    assert_eq!(reporter.state().cache_hits(), 0);
    assert_eq!(reporter.state().total_items(), 2);
    assert_eq!(reporter.state().completed_items(), 2);
    assert_eq!(reporter.state().remaining(), 0);

    let second = translator
        .translate_markdown_with_report(input, &options)
        .await
        .unwrap();

    assert_eq!(second.summary.cache_hits, 2);
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
    assert_eq!(reporter.state().requests_sent(), 0);
    assert_eq!(reporter.state().cache_hits(), 2);
    assert_eq!(reporter.state().total_items(), 2);
    assert_eq!(reporter.state().completed_items(), 2);
    assert_eq!(reporter.state().remaining(), 0);
}
