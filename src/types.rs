use serde::{Deserialize, Serialize};

/// Input document format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentFormat {
    /// Markdown document with AST-driven translation
    Markdown,
    /// Plain text with simple line-based translation
    PlainText,
}

/// Markdown-specific translation options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownOptions {
    /// Translate frontmatter values (default: false)
    pub translate_frontmatter: bool,
    /// Translate content inside fenced code blocks (default: false)
    pub translate_multiline_code: bool,
    /// Translate LaTeX math blocks (default: false)
    pub translate_latex: bool,
    /// Translate link text inside Markdown links (default: true)
    pub translate_link_text: bool,
}

impl Default for MarkdownOptions {
    fn default() -> Self {
        Self {
            translate_frontmatter: false,
            translate_multiline_code: false,
            translate_latex: false,
            translate_link_text: true,
        }
    }
}

/// Batching configuration for translation requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchingOptions {
    /// Maximum items per batch request (default: 20)
    pub max_items_per_batch: usize,
    /// Maximum characters per batch (default: 5000)
    pub max_chars_per_batch: usize,
    /// Context window lines before/after each segment (default: 50)
    pub context_window: usize,
}

impl Default for BatchingOptions {
    fn default() -> Self {
        Self {
            max_items_per_batch: 20,
            max_chars_per_batch: 5000,
            context_window: 50,
        }
    }
}

/// Runtime concurrency and retry settings.
///
/// # Two-Layer Concurrency Model
///
/// This struct governs two distinct concurrency layers that must remain
/// conceptually separate:
///
/// ## Layer 1 — Engine Wave Concurrency (all fields)
/// The engine uses `AdaptiveConcurrency` to decide how many `translate_batch`
/// calls run concurrently (wave size). Fields here control that wave:
///
/// - `max_concurrency`: upper bound on wave size. Also the **single source of
///   truth** for provider-level semaphore limits — see below.
/// - `adaptive_concurrency`: whether wave size grows/shrinks dynamically.
/// - `initial_concurrency`: starting wave size when `adaptive_concurrency` is
///   true. Engine-only; does NOT set provider semaphore limits.
/// - `min_concurrency`: floor for adaptive wave growth. Engine-only.
/// - `max_retries`, `retry_backoff_ms`, `request_timeout_secs`: retry behaviour
///   applied by `translate_with_retries` in the engine.
///
/// ## Layer 2 — Provider Shared Scheduling Limits (`max_concurrency` only)
/// Each provider that performs internal parallel work (DeepLX, OpenAI-compatible)
/// guards that work with a semaphore. The limit for those semaphores is derived
/// from `max_concurrency` — **not** from `initial_concurrency` or any other
/// field. This keeps the two layers aligned: the engine's wave ceiling is also
/// the provider's in-flight ceiling.
///
/// Providers that are fully sequential (GTX, DeepL) are unaffected by this;
/// their `translate_batch` processes items serially regardless of the limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOptions {
    /// Maximum concurrent requests (default: 10).
    ///
    /// This is the **single source of truth** for both:
    /// 1. Engine wave size — how many `translate_batch` calls run in parallel.
    /// 2. Provider semaphore limit — how many internal parallel operations a
    ///    provider (DeepLX, OpenAI-compatible) may have in-flight simultaneously.
    pub max_concurrency: usize,
    /// Whether concurrency should adapt during the run (default: true).
    /// Engine-only; does not affect provider semaphore limits.
    pub adaptive_concurrency: bool,
    /// Initial concurrency when adaptive mode is enabled (default: 5).
    /// Engine-only; does NOT set provider semaphore limits.
    pub initial_concurrency: usize,
    /// Minimum concurrency floor when adaptive mode is enabled (default: 1).
    /// Engine-only; does not affect provider semaphore limits.
    pub min_concurrency: usize,
    /// Maximum retry attempts on failure (default: 3)
    pub max_retries: u32,
    /// Initial retry backoff in milliseconds (default: 300)
    pub retry_backoff_ms: u64,
    /// Request timeout in seconds (default: 60)
    pub request_timeout_secs: u64,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            max_concurrency: 10,
            adaptive_concurrency: true,
            initial_concurrency: 5,
            min_concurrency: 1,
            max_retries: 3,
            retry_backoff_ms: 300,
            request_timeout_secs: 60,
        }
    }
}

/// Cache behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptions {
    /// Enable translation cache (default: true)
    pub enabled: bool,
    /// Cache namespace for isolating different translation contexts (default: "default")
    pub namespace: String,
}

impl Default for CacheOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            namespace: "default".to_string(),
        }
    }
}

/// Complete translation options combining all settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateOptions {
    /// Input document format
    pub format: DocumentFormat,
    /// Source language code (e.g., "en", "auto")
    pub source_lang: String,
    /// Target language code (e.g., "zh", "ja")
    pub target_lang: String,
    /// Markdown-specific options
    pub markdown: MarkdownOptions,
    /// Batching configuration
    pub batching: BatchingOptions,
    /// Runtime concurrency and retry settings
    pub runtime: RuntimeOptions,
    /// Cache behavior settings
    pub cache: CacheOptions,
    /// Custom system prompt for AI providers (optional)
    pub system_prompt: Option<String>,
    /// Custom user prompt template for AI providers (optional)
    pub user_prompt: Option<String>,
}

impl Default for TranslateOptions {
    fn default() -> Self {
        Self {
            format: DocumentFormat::Markdown,
            source_lang: "auto".to_string(),
            target_lang: "zh".to_string(),
            markdown: MarkdownOptions::default(),
            batching: BatchingOptions::default(),
            runtime: RuntimeOptions::default(),
            cache: CacheOptions::default(),
            system_prompt: None,
            user_prompt: None,
        }
    }
}

/// A single item within a translation batch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    /// Unique identifier for this item (used to match responses)
    pub id: usize,
    /// Text content to translate
    pub text: String,
    /// Context lines before this segment (for AI provider prompts)
    pub context_before: Vec<String>,
    /// Context lines after this segment (for AI provider prompts)
    pub context_after: Vec<String>,
}

/// A translation batch request sent to a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateRequest {
    /// Source language code
    pub source_lang: String,
    /// Target language code
    pub target_lang: String,
    /// Items to translate
    pub items: Vec<BatchItem>,
    /// Whether to preserve Markdown formatting markers
    pub preserve_markdown: bool,
    /// Custom system prompt for AI providers
    pub system_prompt: Option<String>,
    /// Custom user prompt template for AI providers
    pub user_prompt: Option<String>,
}

/// A single translated item in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResponseItem {
    /// ID matching the corresponding BatchItem
    pub id: usize,
    /// Translated text
    pub text: String,
}

/// A translation batch response from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateResponse {
    /// Translated items in the same order as the request
    pub items: Vec<TranslateResponseItem>,
}

/// Summary statistics for a translation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationSummary {
    /// Total number of lines/segments in the input
    pub total_lines: usize,
    /// Number of lines/segments that were translated
    pub translated_lines: usize,
    /// Number of cache hits (skipped translation)
    pub cache_hits: usize,
    /// Number of batch requests sent
    pub batches: usize,
    /// Time spent on markdown processing/conversion (AST manipulation)
    pub markdown_processing_time_ms: u64,
    /// Time spent waiting for external translation providers
    pub provider_time_ms: u64,
}

/// Output from a translation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationOutput {
    /// The translated content
    pub content: String,
    /// Summary statistics for this translation
    pub summary: TranslationSummary,
}
