//! # md-translator-rs
//!
//! Translate Markdown documents while preserving structure.
//!
//! This crate provides both a library API ([`MdTranslator`]) and a CLI tool (`mdxlate`)
//! for translating Markdown content through pluggable translation backends.
//!
//! ## Key Features
//!
//! - **AST-driven translation** — parses Markdown via `comrak`, extracts translatable nodes,
//!   translates them, and renders back with 100% structure preservation.
//! - **Pluggable backends** — [`GtxBackend`], [`DeepLXBackend`], [`OpenAICompatBackend`],
//!   [`DeepLBackend`], or implement your own via [`TranslationBackend`].
//! - **Two-tier caching** — in-memory ([`MemoryCache`]) + persistent disk ([`DiskCache`])
//!   via [`TwoTierCache`].
//! - **Request deduplication** — concurrent identical requests are coalesced automatically.
//! - **Multi-provider fallback** — chain providers; on failure, automatically switch to the next.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use md_translator_rs::{GtxBackend, MdTranslator, TranslateOptions, TwoTierCache};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = reqwest::Client::new();
//! let backend = Arc::new(GtxBackend::new(client));
//! let cache = Arc::new(TwoTierCache::new(std::path::PathBuf::from("./.cache"))?);
//! let translator = MdTranslator::new(backend, cache);
//!
//! let options = TranslateOptions::default();
//! let translated = translator
//!     .translate_markdown("# Hello world", &options)
//!     .await?;
//! println!("{translated}");
//! # Ok(())
//! # }
//! ```

pub mod ast;
pub mod cache;
pub mod cli;
pub mod client;
pub mod config;
pub mod dedup;
pub mod engine;
pub mod error;
pub mod fallback;
pub mod markdown;
pub mod provider;
pub mod reporting;
pub mod types;

pub use ast::{MarkdownAst, markdown_parse_options, parse_markdown, parse_markdown_with_options};
pub use cache::{Cache, DiskCache, FileCache, MemoryCache, TwoTierCache};
pub use cli::{Cli, Format, Provider, ProviderKind, run};
pub use client::build_client;
pub use dedup::DeduplicatingBackend;
pub use engine::MdTranslator;
pub use error::{MdTranslatorError, Result};
pub use fallback::{FallbackDecision, MultiProviderBackend};
pub use markdown::{ProtectedMarkdown, protect_markdown, protect_plain_text, restore_markdown};
pub use provider::{
    DeepLBackend, DeepLXBackend, GtxBackend, OpenAICompatBackend, TranslationBackend,
};
pub use reporting::{ReporterState, TranslationReporter, Verbosity};
pub use types::{
    BatchItem, BatchingOptions, CacheOptions, DocumentFormat, MarkdownOptions, RuntimeOptions,
    TranslateOptions, TranslateRequest, TranslateResponse, TranslateResponseItem,
    TranslationOutput, TranslationSummary,
};
