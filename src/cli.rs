use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use directories::ProjectDirs;
use reqwest::Client;

use crate::cache::{MemoryCache, TwoTierCache};
use crate::client::build_client;
use crate::config::Config;
use crate::engine::MdTranslator;
use crate::error::{MdTranslatorError, Result};
use crate::provider::{
    DeepLBackend, DeepLXBackend, GtxBackend, OpenAICompatBackend, TranslationBackend,
};
use crate::reporting::{SummaryReport, TranslationReporter, Verbosity};
use crate::types::{DocumentFormat, TranslateOptions};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ProviderKind {
    Ordinary,
    Ai,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Provider {
    Gtx,
    DeepLx,
    OpenaiCompat,
    DeepL,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Format {
    Markdown,
    PlainText,
}

pub const fn default_model() -> &'static str {
    "gpt-4o-mini"
}

pub const fn default_temperature() -> f32 {
    0.2
}

#[derive(Debug, Parser)]
#[command(name = "mdxlate")]
#[command(about = "Translate Markdown while preserving structure")]
pub struct Cli {
    #[arg(short, long)]
    pub input: PathBuf,

    #[arg(short, long)]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long, default_value = "zh")]
    pub target: String,

    #[arg(long, default_value = "auto")]
    pub source: String,

    #[arg(long, value_enum, default_value = "markdown")]
    pub format: Format,

    #[arg(long, value_enum, default_value = "ordinary")]
    pub provider_kind: ProviderKind,

    #[arg(long, value_enum)]
    pub provider: Provider,

    #[arg(long)]
    pub api_key: Option<String>,

    #[arg(long, env = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,

    #[arg(long, env = "DEEPL_API_KEY")]
    pub deepl_api_key: Option<String>,

    #[arg(long, env = "DEEPLX_ENDPOINT")]
    pub deeplx_endpoint: Option<String>,

    #[arg(long, env = "OPENAI_COMPAT_ENDPOINT")]
    pub openai_endpoint: Option<String>,

    #[arg(long, env = "DEEPL_ENDPOINT")]
    pub deepl_endpoint: Option<String>,

    #[arg(long, env = "MD_TRANSLATOR_MODEL", default_value = default_model())]
    pub model: String,

    #[arg(long, default_value_t = default_temperature())]
    pub temperature: f32,

    #[arg(long)]
    pub translate_frontmatter: bool,

    #[arg(long)]
    pub translate_multiline_code: bool,

    #[arg(long)]
    pub translate_latex: bool,

    #[arg(long)]
    pub no_translate_link_text: bool,

    #[arg(long, default_value_t = 20)]
    pub batch_size: usize,

    #[arg(long, default_value_t = 5000)]
    pub max_chars_per_batch: usize,

    #[arg(long, default_value_t = 50)]
    pub context_window: usize,

    #[arg(long, default_value_t = 6)]
    pub concurrency: usize,

    #[arg(long, default_value_t = 3)]
    pub retries: u32,

    #[arg(long, default_value_t = 300)]
    pub retry_backoff_ms: u64,

    #[arg(long, default_value_t = 60)]
    pub timeout_secs: u64,

    #[arg(long)]
    pub no_cache: bool,

    #[arg(long)]
    pub system_prompt: Option<String>,

    #[arg(long)]
    pub user_prompt: Option<String>,

    #[arg(long, short)]
    pub quiet: bool,

    #[arg(long, short)]
    pub verbose: bool,
}

pub async fn run(cli: Cli) -> Result<()> {
    let config = load_config(cli.config.as_deref(), Config::default_config_path)?;

    let verbosity = if cli.quiet {
        Verbosity::Quiet
    } else if cli.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Normal
    };

    let reporter_concurrency = config
        .as_ref()
        .map(|cfg| cfg.merge_with_cli(&cli).runtime.max_concurrency)
        .unwrap_or(cli.concurrency);
    let reporter = TranslationReporter::new(verbosity, reporter_concurrency);

    let client = build_client();
    let max_concurrency = config
        .as_ref()
        .map(|cfg| cfg.merge_with_cli(&cli).runtime.max_concurrency)
        .unwrap_or(cli.concurrency);
    let backend = build_backend(&cli, config.as_ref(), client.clone(), max_concurrency)?;
    let provider_name = backend.name();

    let project_dirs =
        ProjectDirs::from("top", "newzone", "md-translator-rs").ok_or_else(|| {
            MdTranslatorError::Provider("failed to resolve project dirs for cache".to_string())
        })?;
    let cache = if cli.no_cache {
        Arc::new(MemoryCache::new(
            10_000,
            std::time::Duration::from_secs(7 * 24 * 60 * 60),
        )) as Arc<dyn crate::cache::Cache>
    } else {
        Arc::new(TwoTierCache::new(project_dirs.cache_dir().to_path_buf())?)
            as Arc<dyn crate::cache::Cache>
    };
    let translator = MdTranslator::new(backend, cache).with_reporter(reporter.clone());

    let options = if let Some(config) = &config {
        config.merge_with_cli(&cli)
    } else {
        TranslateOptions {
            source_lang: cli.source.clone(),
            target_lang: cli.target.clone(),
            format: match cli.format {
                Format::Markdown => DocumentFormat::Markdown,
                Format::PlainText => DocumentFormat::PlainText,
            },
            markdown: crate::types::MarkdownOptions {
                translate_frontmatter: cli.translate_frontmatter,
                translate_multiline_code: cli.translate_multiline_code,
                translate_latex: cli.translate_latex,
                translate_link_text: !cli.no_translate_link_text,
            },
            batching: crate::types::BatchingOptions {
                max_items_per_batch: cli.batch_size,
                max_chars_per_batch: cli.max_chars_per_batch,
                context_window: cli.context_window,
            },
            runtime: crate::types::RuntimeOptions {
                max_concurrency: cli.concurrency,
                adaptive_concurrency: crate::types::RuntimeOptions::default().adaptive_concurrency,
                initial_concurrency: crate::types::RuntimeOptions::default().initial_concurrency,
                min_concurrency: crate::types::RuntimeOptions::default().min_concurrency,
                max_retries: cli.retries,
                retry_backoff_ms: cli.retry_backoff_ms,
                request_timeout_secs: cli.timeout_secs,
            },
            cache: crate::types::CacheOptions {
                enabled: !cli.no_cache,
                ..Default::default()
            },
            system_prompt: cli.system_prompt.clone(),
            user_prompt: cli.user_prompt.clone(),
        }
    };

    let output_path = cli
        .output
        .unwrap_or_else(|| default_output_path(&cli.input, &options.target_lang));
    let output = translator
        .translate_file_to(&cli.input, &output_path, &options)
        .await?;

    if reporter.is_quiet() {
    } else {
        reporter.print_summary(&SummaryReport {
            provider_name,
            total_lines: output.summary.total_lines,
            translated_lines: output.summary.translated_lines,
            cache_hits: output.summary.cache_hits,
            batches: output.summary.batches,
            markdown_processing_time_ms: output.summary.markdown_processing_time_ms,
            provider_time_ms: output.summary.provider_time_ms,
        });
        println!("Written to: {}", output_path.display());
    }

    Ok(())
}

fn load_config(
    explicit_config: Option<&Path>,
    default_path: impl FnOnce() -> PathBuf,
) -> Result<Option<Config>> {
    match explicit_config {
        Some(config_path) => Ok(Some(Config::load(config_path)?)),
        None => {
            let config_path = default_path();
            if config_path.exists() {
                Ok(Some(Config::load(&config_path)?))
            } else {
                Ok(None)
            }
        }
    }
}

fn default_output_path(input: &std::path::Path, target_lang: &str) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("translated");
    let ext = input.extension().and_then(|s| s.to_str()).unwrap_or("md");
    input.with_file_name(format!("{stem}.{target_lang}.{ext}"))
}

impl Cli {
    pub fn uses_default_model(&self) -> bool {
        self.model == default_model()
    }

    pub fn uses_default_temperature(&self) -> bool {
        (self.temperature - default_temperature()).abs() < f32::EPSILON
    }
}

fn build_backend(
    cli: &Cli,
    config: Option<&Config>,
    client: Client,
    max_concurrency: usize,
) -> Result<Arc<dyn TranslationBackend>> {
    let provider_config = config.map(|cfg| cfg.resolve_provider_config(cli));
    let backend: Arc<dyn TranslationBackend> = match (cli.provider_kind, cli.provider) {
        (ProviderKind::Ordinary, Provider::Gtx) => Arc::new(GtxBackend::new(client)),
        (ProviderKind::Ordinary, Provider::DeepLx) => {
            let endpoint = provider_config
                .as_ref()
                .and_then(|cfg| cfg.deeplx_endpoint.clone())
                .or_else(|| cli.deeplx_endpoint.clone())
                .ok_or_else(|| {
                    MdTranslatorError::Provider(
                        "--deeplx-endpoint or DEEPLX_ENDPOINT is required for deeplx".to_string(),
                    )
                })?;
            Arc::new(DeepLXBackend::with_concurrency(
                client,
                endpoint,
                max_concurrency,
            ))
        }
        (ProviderKind::Ai, Provider::OpenaiCompat) => {
            let endpoint = provider_config
                .as_ref()
                .and_then(|cfg| cfg.openai_endpoint.clone())
                .or_else(|| cli.openai_endpoint.clone())
                .ok_or_else(|| {
                    MdTranslatorError::Provider(
                        "--openai-endpoint or OPENAI_COMPAT_ENDPOINT is required for openai-compat"
                            .to_string(),
                    )
                })?;
            let api_key = provider_config
                .as_ref()
                .and_then(|cfg| cfg.openai_api_key.clone())
                .or_else(|| cli.api_key.clone())
                .or_else(|| cli.openai_api_key.clone())
                .ok_or_else(|| {
                    MdTranslatorError::Provider(
                        "--api-key or OPENAI_API_KEY is required for openai-compat".to_string(),
                    )
                })?;
            Arc::new(OpenAICompatBackend::with_concurrency(
                client,
                endpoint,
                api_key,
                provider_config
                    .as_ref()
                    .map(|cfg| cfg.model.clone())
                    .unwrap_or_else(|| cli.model.clone()),
                provider_config
                    .as_ref()
                    .map(|cfg| cfg.temperature)
                    .unwrap_or(cli.temperature),
                max_concurrency,
            ))
        }
        (ProviderKind::Ai, Provider::DeepL) => {
            let endpoint = provider_config
                .as_ref()
                .and_then(|cfg| cfg.deepl_endpoint.clone())
                .or_else(|| cli.deepl_endpoint.clone())
                .ok_or_else(|| {
                    MdTranslatorError::Provider(
                        "--deepl-endpoint or DEEPL_ENDPOINT is required for deepl".to_string(),
                    )
                })?;
            let api_key = provider_config
                .as_ref()
                .and_then(|cfg| cfg.deepl_api_key.clone())
                .or_else(|| cli.api_key.clone())
                .or_else(|| cli.deepl_api_key.clone())
                .ok_or_else(|| {
                    MdTranslatorError::Provider(
                        "--api-key or DEEPL_API_KEY is required for deepl".to_string(),
                    )
                })?;
            Arc::new(DeepLBackend::new(client, endpoint, api_key))
        }
        (ProviderKind::Ordinary, Provider::OpenaiCompat)
        | (ProviderKind::Ordinary, Provider::DeepL) => {
            return Err(MdTranslatorError::Provider(
                "ai providers require --provider-kind ai".to_string(),
            ));
        }
        (ProviderKind::Ai, Provider::Gtx) | (ProviderKind::Ai, Provider::DeepLx) => {
            return Err(MdTranslatorError::Provider(
                "ordinary providers require --provider-kind ordinary".to_string(),
            ));
        }
    };

    Ok(backend)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_cli() -> Cli {
        Cli {
            input: PathBuf::from("/tmp/test.md"),
            output: None,
            config: None,
            target: "zh".to_string(),
            source: "auto".to_string(),
            format: Format::Markdown,
            provider_kind: ProviderKind::Ai,
            provider: Provider::OpenaiCompat,
            api_key: None,
            openai_api_key: None,
            deepl_api_key: None,
            deeplx_endpoint: None,
            openai_endpoint: None,
            deepl_endpoint: None,
            model: default_model().to_string(),
            temperature: default_temperature(),
            translate_frontmatter: false,
            translate_multiline_code: false,
            translate_latex: false,
            no_translate_link_text: false,
            batch_size: 20,
            max_chars_per_batch: 5000,
            context_window: 50,
            concurrency: 6,
            retries: 3,
            retry_backoff_ms: 300,
            timeout_secs: 60,
            no_cache: false,
            system_prompt: None,
            user_prompt: None,
            quiet: false,
            verbose: false,
        }
    }

    #[test]
    fn default_detection_helpers_work() {
        let cli = base_cli();
        assert!(cli.uses_default_model());
        assert!(cli.uses_default_temperature());

        let cli = Cli {
            model: "custom-model".to_string(),
            temperature: 0.5,
            ..base_cli()
        };
        assert!(!cli.uses_default_model());
        assert!(!cli.uses_default_temperature());
    }

    #[test]
    fn load_config_uses_explicit_path_when_provided() {
        let dir = std::env::temp_dir().join(format!(
            "md-translator-cli-config-explicit-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("providers.yaml");
        std::fs::write(&config_path, "performance:\n  concurrency: 9\n").unwrap();

        let config = load_config(Some(&config_path), || unreachable!()).unwrap();

        assert_eq!(config.unwrap().performance.concurrency, 9);
    }

    #[test]
    fn load_config_uses_default_path_when_present() {
        let dir = std::env::temp_dir().join(format!(
            "md-translator-cli-config-default-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let config_path = dir.join("providers.yaml");
        std::fs::write(&config_path, "performance:\n  concurrency: 11\n").unwrap();

        let config = load_config(None, || config_path.clone()).unwrap();

        assert_eq!(config.unwrap().performance.concurrency, 11);
    }

    #[test]
    fn load_config_skips_missing_default_path() {
        let missing = std::env::temp_dir().join(format!(
            "md-translator-cli-missing-{}/providers.yaml",
            std::process::id()
        ));
        if missing.exists() {
            std::fs::remove_file(&missing).unwrap();
        }

        let config = load_config(None, || missing.clone()).unwrap();

        assert!(config.is_none());
    }
}
