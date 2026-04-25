use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use reqwest::Client;
use tokio::fs;

use md_translator_rs::{
    CacheOptions, DeepLXBackend, DocumentFormat, MarkdownOptions, MdTranslator, MdTranslatorError,
    RuntimeOptions, TranslateOptions, TranslationBackend, TwoTierCache,
};

fn usage() -> ! {
    eprintln!("Usage: deeplx_export <INPUT> <OUTPUT>");
    eprintln!("  env DEEPLX_ENDPOINT must be set");
    std::process::exit(1);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        usage();
    }

    let endpoint = std::env::var("DEEPLX_ENDPOINT")?;

    let input = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    let cache_root = std::env::temp_dir().join("md-translator-rs-deeplx-export-cache");

    let client = Client::builder()
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let backend = Arc::new(DeepLXBackend::new(client, endpoint));
    let cache = Arc::new(TwoTierCache::new(cache_root)?);
    let translator = MdTranslator::new(backend, cache);

    let options = TranslateOptions {
        format: DocumentFormat::Markdown,
        source_lang: "auto".to_string(),
        target_lang: "zh".to_string(),
        markdown: MarkdownOptions::default(),
        batching: md_translator_rs::BatchingOptions {
            max_items_per_batch: 256,
            max_chars_per_batch: 200_000,
            context_window: 0,
        },
        runtime: RuntimeOptions {
            max_concurrency: 1,
            adaptive_concurrency: false,
            initial_concurrency: 1,
            min_concurrency: 1,
            max_retries: 1,
            retry_backoff_ms: 100,
            request_timeout_secs: 12,
        },
        cache: CacheOptions {
            enabled: true,
            namespace: "deeplx-export".to_string(),
        },
        system_prompt: Some(
            "You are a Markdown translator. Preserve all placeholder tokens that look like __MDT_*__ exactly, without translating, reformatting, inserting spaces, or changing punctuation.".to_string(),
        ),
        user_prompt: Some(
            "Translate from {source} to {target}. Preserve Markdown structure and any placeholder tokens that look like __MDT_*__ exactly as-is. Return translation only.\n\n{text}".to_string(),
        ),
    };

    let started = Instant::now();
    let translation = translator
        .translate_file_to(&input, &output, &options)
        .await;
    let (output_report, export_mode) = match translation {
        Ok(report) => (report, "markdown"),
        Err(MdTranslatorError::InvalidResponse(message))
            if message.contains("protected placeholders") =>
        {
            let fallback_input = fs::read_to_string(&input).await?;
            let fallback_client = Client::builder()
                .connect_timeout(std::time::Duration::from_secs(3))
                .timeout(std::time::Duration::from_secs(12))
                .build()?;
            let fallback_backend =
                DeepLXBackend::new(fallback_client, std::env::var("DEEPLX_ENDPOINT")?);
            let fallback_text = fallback_backend
                .translate_batch(md_translator_rs::TranslateRequest {
                    source_lang: "auto".to_string(),
                    target_lang: "zh".to_string(),
                    items: vec![md_translator_rs::BatchItem {
                        id: 0,
                        text: fallback_input,
                        context_before: Vec::new(),
                        context_after: Vec::new(),
                    }],
                    preserve_markdown: false,
                    system_prompt: Some(
                        "You are a translation engine. Translate the content faithfully into Chinese and return plain translated text only."
                            .to_string(),
                    ),
                    user_prompt: Some(
                        "Translate from {source} to {target}. Return translated text only.\n\n{text}"
                            .to_string(),
                    ),
                })
                .await?
                .items
                .into_iter()
                .next()
                .map(|item| item.text)
                .unwrap_or_default();

            fs::write(&output, fallback_text.as_bytes()).await?;
            (
                md_translator_rs::TranslationOutput {
                    content: fallback_text,
                    summary: md_translator_rs::TranslationSummary {
                        total_lines: 1,
                        translated_lines: 1,
                        cache_hits: 0,
                        batches: 1,
                        markdown_processing_time_ms: 0,
                        provider_time_ms: 0,
                    },
                },
                "single-request-plain-text-fallback",
            )
        }
        Err(error) => return Err(error.into()),
    };
    let elapsed = started.elapsed();
    let output_metadata = fs::metadata(&output).await?;

    println!("input={}", input.display());
    println!("output={}", output.display());
    println!("export_mode={}", export_mode);
    println!("elapsed_ms={}", elapsed.as_millis());
    println!(
        "translated_lines={}",
        output_report.summary.translated_lines
    );
    println!("cache_hits={}", output_report.summary.cache_hits);
    println!("batches={}", output_report.summary.batches);
    println!("output_bytes={}", output_metadata.len());

    Ok(())
}
