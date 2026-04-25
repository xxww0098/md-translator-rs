//! YAML configuration system for md-translator-rs.
//!
//! Supports loading from explicit path or default path (`~/.config/md-translator-rs/providers.yaml`).
//! CLI arguments override YAML config (standard precedence).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use crate::types::{
    BatchingOptions, CacheOptions, DocumentFormat, MarkdownOptions, RuntimeOptions,
    TranslateOptions,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProviderConfig {
    pub deeplx_endpoint: Option<String>,
    pub openai_endpoint: Option<String>,
    pub openai_api_key: Option<String>,
    pub deepl_endpoint: Option<String>,
    pub deepl_api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
}

/// DeepLX provider settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeepLXProviderConfig {
    /// DeepLX endpoint URL (required when using deep-lx provider).
    pub deeplx_endpoint: Option<String>,
}

/// OpenAI-compatible provider settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiProviderConfig {
    /// OpenAI-compatible endpoint URL.
    pub openai_endpoint: Option<String>,
    /// API key for the OpenAI-compatible provider.
    pub api_key: Option<String>,
    /// Model name (default: gpt-4o-mini).
    pub model: Option<String>,
    /// Temperature (default: 0.2).
    pub temperature: Option<f32>,
}

/// DeepL native provider settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeepLProviderConfig {
    /// DeepL endpoint URL.
    pub deepl_endpoint: Option<String>,
    /// DeepL API key.
    pub api_key: Option<String>,
}

/// Performance and batching settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of items per batch (default: 20).
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Maximum characters per batch (default: 5000).
    #[serde(default = "default_max_chars_per_batch")]
    pub max_chars_per_batch: usize,
    /// Context window size (default: 50).
    #[serde(default = "default_context_window")]
    pub context_window: usize,
    /// Maximum concurrency (default: 6).
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    /// Whether concurrency should adapt during a run (default: true).
    #[serde(default = "default_adaptive_concurrency")]
    pub adaptive_concurrency: bool,
    /// Initial concurrency when adaptive mode is enabled (default: 2).
    #[serde(default = "default_initial_concurrency")]
    pub initial_concurrency: usize,
    /// Minimum concurrency floor when adaptive mode is enabled (default: 1).
    #[serde(default = "default_min_concurrency")]
    pub min_concurrency: usize,
    /// Maximum retries (default: 3).
    #[serde(default = "default_retries")]
    pub retries: u32,
    /// Retry backoff in milliseconds (default: 300).
    #[serde(default = "default_retry_backoff_ms")]
    pub retry_backoff_ms: u64,
    /// Request timeout in seconds (default: 60).
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: default_batch_size(),
            max_chars_per_batch: default_max_chars_per_batch(),
            context_window: default_context_window(),
            concurrency: default_concurrency(),
            adaptive_concurrency: default_adaptive_concurrency(),
            initial_concurrency: default_initial_concurrency(),
            min_concurrency: default_min_concurrency(),
            retries: default_retries(),
            retry_backoff_ms: default_retry_backoff_ms(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// Cache settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether cache is enabled (default: true).
    #[serde(default = "default_cache_enabled")]
    pub enabled: bool,
    /// Cache namespace (default: "default").
    #[serde(default = "default_cache_namespace")]
    pub namespace: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: default_cache_enabled(),
            namespace: default_cache_namespace(),
        }
    }
}

/// Markdown-specific translation options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownConfig {
    /// Translate frontmatter (default: false).
    #[serde(default)]
    pub translate_frontmatter: bool,
    /// Translate multiline code blocks (default: false).
    #[serde(default)]
    pub translate_multiline_code: bool,
    /// Translate LaTeX math (default: false).
    #[serde(default)]
    pub translate_latex: bool,
    /// Translate link text (default: true).
    #[serde(default = "default_translate_link_text")]
    pub translate_link_text: bool,
}

impl Default for MarkdownConfig {
    fn default() -> Self {
        Self {
            translate_frontmatter: false,
            translate_multiline_code: false,
            translate_latex: false,
            translate_link_text: default_translate_link_text(),
        }
    }
}

/// Root configuration structure loaded from YAML.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// DeepLX provider settings.
    #[serde(default)]
    pub deeplx: DeepLXProviderConfig,
    /// OpenAI-compatible provider settings.
    #[serde(default)]
    pub llm: AiProviderConfig,
    /// DeepL provider settings.
    #[serde(default)]
    pub deepl: DeepLProviderConfig,
    /// Performance settings.
    #[serde(default)]
    pub performance: PerformanceConfig,
    /// Cache settings.
    #[serde(default)]
    pub cache: CacheConfig,
    /// Markdown settings.
    #[serde(default)]
    pub markdown: MarkdownConfig,
}

impl Config {
    /// Load configuration from a YAML file.
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::Io(path.as_ref().display().to_string(), e))?;
        serde_yaml::from_str(&content).map_err(ConfigError::Parse)
    }

    /// Returns the default config file path: `~/.config/md-translator-rs/providers.yaml`.
    pub fn default_config_path() -> PathBuf {
        let home = dirs::home_dir().expect("cannot resolve home directory");
        home.join(".config")
            .join("md-translator-rs")
            .join("providers.yaml")
    }

    /// Returns the configured DeepL endpoint.
    pub fn deepl_endpoint(&self) -> Option<&str> {
        self.deepl.deepl_endpoint.as_deref()
    }

    /// Returns the configured DeepL API key.
    pub fn deepl_api_key(&self) -> Option<&str> {
        self.deepl.api_key.as_deref()
    }

    /// Merge CLI arguments into this config, returning a `TranslateOptions`.
    ///
    /// CLI arguments take precedence over YAML values (standard override behavior).
    /// Missing CLI fields retain YAML values or defaults.
    pub fn merge_with_cli(&self, cli: &Cli) -> TranslateOptions {
        TranslateOptions {
            format: match cli.format {
                crate::cli::Format::Markdown => DocumentFormat::Markdown,
                crate::cli::Format::PlainText => DocumentFormat::PlainText,
            },
            source_lang: cli.source.clone(),
            target_lang: cli.target.clone(),
            markdown: MarkdownOptions {
                translate_frontmatter: cli.translate_frontmatter
                    || self.markdown.translate_frontmatter,
                translate_multiline_code: cli.translate_multiline_code
                    || self.markdown.translate_multiline_code,
                translate_latex: cli.translate_latex || self.markdown.translate_latex,
                // CLI flag --no-translate-link_text means !translate_link_text
                translate_link_text: !cli.no_translate_link_text
                    && self.markdown.translate_link_text,
            },
            batching: BatchingOptions {
                max_items_per_batch: if cli.batch_size != default_batch_size() {
                    cli.batch_size
                } else {
                    self.performance.batch_size
                },
                max_chars_per_batch: if cli.max_chars_per_batch != default_max_chars_per_batch() {
                    cli.max_chars_per_batch
                } else {
                    self.performance.max_chars_per_batch
                },
                context_window: if cli.context_window != default_context_window() {
                    cli.context_window
                } else {
                    self.performance.context_window
                },
            },
            runtime: RuntimeOptions {
                max_concurrency: if cli.concurrency != default_concurrency() {
                    cli.concurrency
                } else {
                    self.performance.concurrency
                },
                adaptive_concurrency: self.performance.adaptive_concurrency,
                initial_concurrency: self.performance.initial_concurrency,
                min_concurrency: self.performance.min_concurrency,
                max_retries: if cli.retries != default_retries() {
                    cli.retries
                } else {
                    self.performance.retries
                },
                retry_backoff_ms: if cli.retry_backoff_ms != default_retry_backoff_ms() {
                    cli.retry_backoff_ms
                } else {
                    self.performance.retry_backoff_ms
                },
                request_timeout_secs: if cli.timeout_secs != default_timeout_secs() {
                    cli.timeout_secs
                } else {
                    self.performance.timeout_secs
                },
            },
            cache: CacheOptions {
                enabled: !cli.no_cache && self.cache.enabled,
                namespace: self.cache.namespace.clone(),
            },
            system_prompt: cli.system_prompt.clone(),
            user_prompt: cli.user_prompt.clone(),
        }
    }

    pub fn resolve_provider_config(&self, cli: &Cli) -> ResolvedProviderConfig {
        ResolvedProviderConfig {
            deeplx_endpoint: cli
                .deeplx_endpoint
                .clone()
                .or_else(|| self.deeplx.deeplx_endpoint.clone()),
            openai_endpoint: cli
                .openai_endpoint
                .clone()
                .or_else(|| self.llm.openai_endpoint.clone()),
            openai_api_key: cli
                .api_key
                .clone()
                .or_else(|| cli.openai_api_key.clone())
                .or_else(|| self.llm.api_key.clone()),
            deepl_endpoint: cli
                .deepl_endpoint
                .clone()
                .or_else(|| self.deepl_endpoint().map(ToOwned::to_owned)),
            deepl_api_key: cli
                .api_key
                .clone()
                .or_else(|| cli.deepl_api_key.clone())
                .or_else(|| self.deepl_api_key().map(ToOwned::to_owned)),
            model: if cli.uses_default_model() {
                self.llm.model.clone().unwrap_or_else(|| cli.model.clone())
            } else {
                cli.model.clone()
            },
            temperature: if cli.uses_default_temperature() {
                self.llm.temperature.unwrap_or(cli.temperature)
            } else {
                cli.temperature
            },
        }
    }
}

/// Configuration loading or parsing errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file '{0}': {1}")]
    Io(String, #[source] std::io::Error),
    #[error("failed to parse config: {0}")]
    Parse(#[source] serde_yaml::Error),
}

fn default_batch_size() -> usize {
    20
}
fn default_max_chars_per_batch() -> usize {
    5000
}
fn default_context_window() -> usize {
    50
}
fn default_concurrency() -> usize {
    6
}
fn default_adaptive_concurrency() -> bool {
    true
}
fn default_initial_concurrency() -> usize {
    2
}
fn default_min_concurrency() -> usize {
    1
}
fn default_retries() -> u32 {
    3
}
fn default_retry_backoff_ms() -> u64 {
    300
}
fn default_timeout_secs() -> u64 {
    60
}
fn default_cache_enabled() -> bool {
    true
}
fn default_cache_namespace() -> String {
    "default".to_string()
}
fn default_translate_link_text() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn write_temp_file(content: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("md-translator-test");
        std::fs::create_dir_all(&dir).unwrap();
        let id = FILE_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = dir.join(format!("cfg-{}.yaml", id));
        std::fs::write(&path, content).unwrap();
        path
    }

    const EXAMPLE_YAML: &str = "deeplx:
  deeplx_endpoint: \"https://api.deeplx.org/test/translate\"

llm:
  openai_endpoint: \"https://api.openai.com/v1/chat/completions\"
  api_key: \"sk-test-key\"
  model: \"gpt-4o-mini\"
  temperature: 0.3

deepl:
  deepl_endpoint: \"https://api-free.deepl.com/v2/translate\"
  api_key: \"deepl-test-key\"

performance:
  batch_size: 25
  max_chars_per_batch: 6000
  context_window: 100
  concurrency: 8
  retries: 5
  retry_backoff_ms: 500
  timeout_secs: 120

cache:
  enabled: true
  namespace: \"prod\"

markdown:
  translate_frontmatter: true
  translate_multiline_code: true
  translate_latex: false
  translate_link_text: false
";

    #[test]
    fn load_yaml() {
        let path = write_temp_file(EXAMPLE_YAML);
        let cfg = Config::load(&path).expect("config should load");
        assert_eq!(
            cfg.deeplx.deeplx_endpoint.as_deref(),
            Some("https://api.deeplx.org/test/translate")
        );
        assert_eq!(
            cfg.llm.openai_endpoint.as_deref(),
            Some("https://api.openai.com/v1/chat/completions")
        );
        assert_eq!(cfg.llm.api_key.as_deref(), Some("sk-test-key"));
        assert_eq!(cfg.llm.model.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(cfg.llm.temperature, Some(0.3));
        assert_eq!(
            cfg.deepl.deepl_endpoint.as_deref(),
            Some("https://api-free.deepl.com/v2/translate")
        );
        assert_eq!(cfg.deepl.api_key.as_deref(), Some("deepl-test-key"));
        assert_eq!(cfg.performance.batch_size, 25);
        assert_eq!(cfg.performance.max_chars_per_batch, 6000);
        assert_eq!(cfg.performance.context_window, 100);
        assert_eq!(cfg.performance.concurrency, 8);
        assert_eq!(cfg.performance.retries, 5);
        assert_eq!(cfg.performance.retry_backoff_ms, 500);
        assert_eq!(cfg.performance.timeout_secs, 120);
        assert!(cfg.cache.enabled);
        assert_eq!(cfg.cache.namespace, "prod");
        assert!(cfg.markdown.translate_frontmatter);
        assert!(cfg.markdown.translate_multiline_code);
        assert!(!cfg.markdown.translate_latex);
        assert!(!cfg.markdown.translate_link_text);
    }

    #[test]
    fn load_yaml_partial() {
        let yaml = "llm:
  api_key: \"sk-minimal\"
";
        let path = write_temp_file(yaml);
        let cfg = Config::load(&path).expect("config should load");
        assert!(cfg.llm.api_key.is_some());
        assert!(cfg.deeplx.deeplx_endpoint.is_none());
        assert!(cfg.deepl.deepl_endpoint.is_none());
        assert_eq!(cfg.performance.batch_size, 20);
        assert_eq!(cfg.performance.concurrency, 6);
        assert!(cfg.cache.enabled);
        assert_eq!(cfg.cache.namespace, "default");
    }

    #[test]
    fn load_yaml_missing_file() {
        let result = Config::load("/nonexistent/path/config.yaml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io(_, _)));
    }

    #[test]
    fn load_yaml_invalid_syntax() {
        let yaml = "invalid: [yaml: syntax";
        let path = write_temp_file(yaml);
        let result = Config::load(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Parse(_)));
    }

    #[test]
    fn merge_cli_overrides_yaml() {
        let yaml = "performance:
  batch_size: 25
  concurrency: 8
markdown:
  translate_frontmatter: true
";
        let path = write_temp_file(yaml);
        let cfg = Config::load(&path).expect("config should load");

        let cli = Cli {
            input: PathBuf::from("/tmp/test.md"),
            output: None,
            config: None,
            target: "ja".to_string(),
            source: "en".to_string(),
            format: crate::cli::Format::Markdown,
            provider_kind: crate::cli::ProviderKind::Ai,
            provider: crate::cli::Provider::OpenaiCompat,
            api_key: None,
            openai_api_key: None,
            deepl_api_key: None,
            deeplx_endpoint: None,
            openai_endpoint: None,
            deepl_endpoint: None,
            model: "gpt-4o-mini".to_string(),
            temperature: 0.2,
            translate_frontmatter: false,
            translate_multiline_code: false,
            translate_latex: false,
            no_translate_link_text: false,
            batch_size: 30,
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
        };

        let opts = cfg.merge_with_cli(&cli);

        assert_eq!(opts.batching.max_items_per_batch, 30);
        assert_eq!(opts.batching.context_window, 50);
        assert!(opts.markdown.translate_frontmatter);
        assert_eq!(opts.target_lang, "ja");
        assert_eq!(opts.source_lang, "en");
    }

    #[test]
    fn merge_cli_no_config_file() {
        let cfg = Config::default();

        let cli = Cli {
            input: PathBuf::from("/tmp/test.md"),
            output: None,
            config: None,
            target: "zh".to_string(),
            source: "auto".to_string(),
            format: crate::cli::Format::Markdown,
            provider_kind: crate::cli::ProviderKind::Ordinary,
            provider: crate::cli::Provider::Gtx,
            api_key: None,
            openai_api_key: None,
            deepl_api_key: None,
            deeplx_endpoint: None,
            openai_endpoint: None,
            deepl_endpoint: None,
            model: "gpt-4o-mini".to_string(),
            temperature: 0.2,
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
        };

        let opts = cfg.merge_with_cli(&cli);

        assert_eq!(opts.target_lang, "zh");
        assert_eq!(opts.source_lang, "auto");
        assert_eq!(opts.batching.max_items_per_batch, 20);
        assert!(opts.cache.enabled);
    }

    #[test]
    fn default_config_path_format() {
        let path = Config::default_config_path();
        assert!(path.ends_with("providers.yaml"));
        assert!(path.to_string_lossy().contains(".config"));
        assert!(path.to_string_lossy().contains("md-translator-rs"));
    }

    #[test]
    fn cache_disabled_via_cli() {
        let cfg = Config::default();

        let cli = Cli {
            input: PathBuf::from("/tmp/test.md"),
            output: None,
            config: None,
            target: "zh".to_string(),
            source: "auto".to_string(),
            format: crate::cli::Format::Markdown,
            provider_kind: crate::cli::ProviderKind::Ordinary,
            provider: crate::cli::Provider::Gtx,
            api_key: None,
            openai_api_key: None,
            deepl_api_key: None,
            deeplx_endpoint: None,
            openai_endpoint: None,
            deepl_endpoint: None,
            model: "gpt-4o-mini".to_string(),
            temperature: 0.2,
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
            no_cache: true,
            system_prompt: None,
            user_prompt: None,
            quiet: false,
            verbose: false,
        };

        let opts = cfg.merge_with_cli(&cli);
        assert!(!opts.cache.enabled);

        let yaml = "cache:
  enabled: true
";
        let path = write_temp_file(yaml);
        let cfg = Config::load(&path).expect("config should load");
        let opts = cfg.merge_with_cli(&cli);
        assert!(!opts.cache.enabled);
    }

    #[test]
    fn translate_link_text_no_override() {
        let yaml = "markdown:
  translate_link_text: false
";
        let path = write_temp_file(yaml);
        let cfg = Config::load(&path).expect("config should load");

        let cli = Cli {
            input: PathBuf::from("/tmp/test.md"),
            output: None,
            config: None,
            target: "zh".to_string(),
            source: "auto".to_string(),
            format: crate::cli::Format::Markdown,
            provider_kind: crate::cli::ProviderKind::Ordinary,
            provider: crate::cli::Provider::Gtx,
            api_key: None,
            openai_api_key: None,
            deepl_api_key: None,
            deeplx_endpoint: None,
            openai_endpoint: None,
            deepl_endpoint: None,
            model: "gpt-4o-mini".to_string(),
            temperature: 0.2,
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
        };

        let opts = cfg.merge_with_cli(&cli);
        assert!(!opts.markdown.translate_link_text);

        let cli = Cli {
            no_translate_link_text: true,
            ..cli
        };
        let opts = cfg.merge_with_cli(&cli);
        assert!(!opts.markdown.translate_link_text);
    }

    #[test]
    fn resolve_provider_config_uses_yaml_when_cli_uses_defaults() {
        let yaml = "deeplx:
  deeplx_endpoint: \"https://deeplx.example.test/translate\"
llm:
  openai_endpoint: \"https://openai.example.test/v1/chat/completions\"
  api_key: \"yaml-openai-key\"
  model: \"yaml-model\"
  temperature: 0.7
deepl:
  deepl_endpoint: \"https://deepl.example.test/v2/translate\"
  api_key: \"yaml-deepl-key\"
";
        let path = write_temp_file(yaml);
        let cfg = Config::load(&path).expect("config should load");

        let cli = Cli {
            input: PathBuf::from("/tmp/test.md"),
            output: None,
            config: None,
            target: "zh".to_string(),
            source: "auto".to_string(),
            format: crate::cli::Format::Markdown,
            provider_kind: crate::cli::ProviderKind::Ai,
            provider: crate::cli::Provider::OpenaiCompat,
            api_key: None,
            openai_api_key: None,
            deepl_api_key: None,
            deeplx_endpoint: None,
            openai_endpoint: None,
            deepl_endpoint: None,
            model: crate::cli::default_model().to_string(),
            temperature: crate::cli::default_temperature(),
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
        };

        let provider = cfg.resolve_provider_config(&cli);
        assert_eq!(
            provider.deeplx_endpoint.as_deref(),
            Some("https://deeplx.example.test/translate")
        );
        assert_eq!(
            provider.openai_endpoint.as_deref(),
            Some("https://openai.example.test/v1/chat/completions")
        );
        assert_eq!(provider.openai_api_key.as_deref(), Some("yaml-openai-key"));
        assert_eq!(
            provider.deepl_endpoint.as_deref(),
            Some("https://deepl.example.test/v2/translate")
        );
        assert_eq!(provider.deepl_api_key.as_deref(), Some("yaml-deepl-key"));
        assert_eq!(provider.model, "yaml-model");
        assert_eq!(provider.temperature, 0.7);
    }

    #[test]
    fn resolve_provider_config_prefers_cli_values() {
        let yaml = "llm:
  openai_endpoint: \"https://yaml.example/v1/chat/completions\"
  api_key: \"yaml-key\"
  model: \"yaml-model\"
  temperature: 0.7
";
        let path = write_temp_file(yaml);
        let cfg = Config::load(&path).expect("config should load");

        let cli = Cli {
            input: PathBuf::from("/tmp/test.md"),
            output: None,
            config: None,
            target: "zh".to_string(),
            source: "auto".to_string(),
            format: crate::cli::Format::Markdown,
            provider_kind: crate::cli::ProviderKind::Ai,
            provider: crate::cli::Provider::OpenaiCompat,
            api_key: Some("cli-key".to_string()),
            openai_api_key: None,
            deepl_api_key: None,
            deeplx_endpoint: None,
            openai_endpoint: Some("https://cli.example/v1/chat/completions".to_string()),
            deepl_endpoint: None,
            model: "cli-model".to_string(),
            temperature: 0.1,
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
        };

        let provider = cfg.resolve_provider_config(&cli);
        assert_eq!(
            provider.openai_endpoint.as_deref(),
            Some("https://cli.example/v1/chat/completions")
        );
        assert_eq!(provider.openai_api_key.as_deref(), Some("cli-key"));
        assert_eq!(provider.model, "cli-model");
        assert_eq!(provider.temperature, 0.1);
    }
}

mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var("USERPROFILE")
                .ok()
                .map(PathBuf::from)
                .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        }
        #[cfg(not(windows))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
    }
}
