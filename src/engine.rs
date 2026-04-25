use std::any::Any;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use blake3::Hasher;
use futures::future::join_all;
use rand::Rng;
use serde_json::json;
use tokio::fs;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::ast::{AstTranslationPipeline, markdown_parse_options};
use crate::cache::Cache;
use crate::dedup::DeduplicatingBackend;
use crate::error::{MdTranslatorError, Result};
use crate::fallback::MultiProviderBackend;
use crate::markdown::{
    ProtectedMarkdown, protect_plain_text, restore_markdown, validate_placeholders,
};
use crate::provider::TranslationBackend;
use crate::reporting::TranslationReporter;
use crate::types::{
    BatchItem, DocumentFormat, RuntimeOptions, TranslateOptions, TranslateRequest,
    TranslationOutput, TranslationSummary,
};

/// Core translation engine.
///
/// `MdTranslator` orchestrates the full translation pipeline:
/// parsing → extraction → batching → translation → cache → rendering.
///
/// Construct one via [`MdTranslator::new`] with any [`TranslationBackend`] and [`Cache`]
/// implementation, then call [`translate_markdown`](MdTranslator::translate_markdown) or
/// [`translate_file_to`](MdTranslator::translate_file_to).
#[derive(Clone)]
pub struct MdTranslator {
    backend: Arc<dyn TranslationBackend>,
    cache: Arc<dyn Cache>,
    reporter: Option<TranslationReporter>,
}

impl MdTranslator {
    pub fn new(backend: Arc<dyn TranslationBackend>, cache: Arc<dyn Cache>) -> Self {
        let backend: Arc<dyn TranslationBackend> = if let Some(multi) =
            (backend.as_ref() as &dyn Any).downcast_ref::<MultiProviderBackend>()
        {
            Arc::new(multi.with_deduplicated_providers())
        } else {
            Arc::new(DeduplicatingBackend::new(backend))
        };
        Self {
            backend,
            cache,
            reporter: None,
        }
    }

    pub fn with_reporter(mut self, reporter: TranslationReporter) -> Self {
        self.reporter = Some(reporter);
        self
    }

    pub async fn translate_markdown(
        &self,
        input: &str,
        options: &TranslateOptions,
    ) -> Result<String> {
        Ok(self
            .translate_markdown_with_report(input, options)
            .await?
            .content)
    }

    pub async fn translate_markdown_with_report(
        &self,
        input: &str,
        options: &TranslateOptions,
    ) -> Result<TranslationOutput> {
        self.translate_markdown_ast(input, options).await
    }

    pub async fn translate_plain_text(
        &self,
        input: &str,
        options: &TranslateOptions,
    ) -> Result<String> {
        Ok(self
            .translate_plain_text_with_report(input, options)
            .await?
            .content)
    }

    pub async fn translate_plain_text_with_report(
        &self,
        input: &str,
        options: &TranslateOptions,
    ) -> Result<TranslationOutput> {
        let protected = protect_plain_text(input);
        self.translate_protected(protected, options).await
    }

    pub async fn translate_string(
        &self,
        input: &str,
        options: &TranslateOptions,
    ) -> Result<TranslationOutput> {
        match options.format {
            DocumentFormat::Markdown => self.translate_markdown_with_report(input, options).await,
            DocumentFormat::PlainText => {
                self.translate_plain_text_with_report(input, options).await
            }
        }
    }

    pub async fn translate_file_to<P, Q>(
        &self,
        input: P,
        output: Q,
        options: &TranslateOptions,
    ) -> Result<TranslationOutput>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let input_path = input.as_ref();
        let output_path = output.as_ref();

        let bytes = fs::read(input_path).await.map_err(|err| {
            MdTranslatorError::Io(std::io::Error::new(
                err.kind(),
                format!("failed to read input file {}: {err}", input_path.display()),
            ))
        })?;
        let source = String::from_utf8(bytes)?;

        let translated = self.translate_string(&source, options).await?;
        fs::write(output_path, translated.content.as_bytes())
            .await
            .map_err(|err| {
                MdTranslatorError::Io(std::io::Error::new(
                    err.kind(),
                    format!(
                        "failed to write output file {}: {err}",
                        output_path.display()
                    ),
                ))
            })?;

        Ok(translated)
    }

    async fn translate_protected(
        &self,
        protected: ProtectedMarkdown,
        options: &TranslateOptions,
    ) -> Result<TranslationOutput> {
        self.reset_reporter();
        let markdown_start = Instant::now();
        let lines = protected
            .text
            .split('\n')
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let segments = collect_segments(&lines, options.batching.context_window);
        let markdown_parse_time = markdown_start.elapsed();

        let provider_start = Instant::now();
        let (translated_lines, mut summary) =
            self.translate_lines(lines, segments, options).await?;
        let provider_time = provider_start.elapsed();

        let render_start = Instant::now();
        let translated_text = translated_lines.join("\n");

        if protected.has_placeholders() && !validate_placeholders(&translated_text, &protected) {
            return Err(MdTranslatorError::InvalidResponse(
                "provider response mutated or removed protected placeholders".to_string(),
            ));
        }

        let content = restore_markdown(&translated_text, &protected);
        let render_time = render_start.elapsed();

        summary.markdown_processing_time_ms =
            (markdown_parse_time + render_time).as_millis() as u64;
        summary.provider_time_ms = provider_time.as_millis() as u64;

        Ok(TranslationOutput { content, summary })
    }

    async fn translate_markdown_ast(
        &self,
        input: &str,
        options: &TranslateOptions,
    ) -> Result<TranslationOutput> {
        self.reset_reporter();
        let markdown_start = Instant::now();
        let pipeline =
            AstTranslationPipeline::parse(input, markdown_parse_options(), &options.markdown);

        if pipeline.nodes.is_empty() {
            return Ok(TranslationOutput {
                content: input.to_string(),
                summary: TranslationSummary {
                    total_lines: 0,
                    translated_lines: 0,
                    cache_hits: 0,
                    batches: 0,
                    markdown_processing_time_ms: markdown_start.elapsed().as_millis() as u64,
                    provider_time_ms: 0,
                },
            });
        }

        let markdown_parse_time = markdown_start.elapsed();

        let provider_start = Instant::now();
        let (translations, mut summary) = self.translate_ast_nodes(&pipeline, options).await?;
        let provider_time = provider_start.elapsed();

        let render_start = Instant::now();
        let content = pipeline
            .render_with_translations(&options.markdown, &translations)
            .map_err(|err| MdTranslatorError::InvalidResponse(err.to_string()))?;
        let render_time = render_start.elapsed();

        summary.markdown_processing_time_ms =
            (markdown_parse_time + render_time).as_millis() as u64;
        summary.provider_time_ms = provider_time.as_millis() as u64;

        Ok(TranslationOutput { content, summary })
    }

    async fn translate_ast_nodes(
        &self,
        pipeline: &AstTranslationPipeline,
        options: &TranslateOptions,
    ) -> Result<(HashMap<usize, String>, TranslationSummary)> {
        let fingerprint = json!({
            "cache_schema": 3,
            "format": "markdown-ast",
            "source": options.source_lang,
            "target": options.target_lang,
            "provider": {
                "name": self.backend.name(),
                "fingerprint": self.backend.cache_fingerprint(),
            },
            "markdown": {
                "translate_frontmatter": options.markdown.translate_frontmatter,
                "translate_multiline_code": options.markdown.translate_multiline_code,
                "translate_latex": options.markdown.translate_latex,
                "translate_link_text": options.markdown.translate_link_text,
            },
            "batching": {
                "max_items_per_batch": options.batching.max_items_per_batch,
                "max_chars_per_batch": options.batching.max_chars_per_batch,
            },
            "system_prompt": options.system_prompt,
            "user_prompt": options.user_prompt,
        })
        .to_string();

        let mut summary = TranslationSummary {
            total_lines: pipeline.nodes.len(),
            translated_lines: 0,
            cache_hits: 0,
            batches: 0,
            markdown_processing_time_ms: 0,
            provider_time_ms: 0,
        };
        let mut translations = std::collections::HashMap::with_capacity(pipeline.nodes.len());
        let mut pending = Vec::new();

        self.report_total_items(summary.total_lines);
        self.report_cache_hits(0);
        self.report_completed(0);

        for node in &pipeline.nodes {
            let cache_key = hash_key(&[
                &options.cache.namespace,
                self.backend.name(),
                &fingerprint,
                &node.text,
            ]);

            if options.cache.enabled
                && let Some(cached) = self.cache.get(&cache_key).await?
            {
                translations.insert(node.id, cached);
                summary.cache_hits += 1;
                summary.translated_lines += 1;
                self.report_cache_hit();
                self.report_completed(1);
                self.report_progress();
                continue;
            }

            pending.push(PendingAstNode {
                id: node.id,
                text: node.text.clone(),
                cache_key,
            });
        }

        if pending.is_empty() {
            return Ok((translations, summary));
        }

        let batches = build_ast_batches(
            &pending,
            options.batching.max_items_per_batch,
            options.batching.max_chars_per_batch,
        );
        summary.batches = batches.len();
        self.report_provider();

        if let Some(multi) =
            (self.backend.as_ref() as &dyn Any).downcast_ref::<MultiProviderBackend>()
        {
            let requests = batches
                .iter()
                .map(|batch| ast_request_from_batch(batch, options))
                .collect::<Vec<_>>();
            let responses = multi
                .translate_document_requests(&requests, &options.runtime)
                .await?;

            for (batch, response) in batches.into_iter().zip(responses) {
                if response.items.len() != batch.len() {
                    return Err(MdTranslatorError::BatchMismatch {
                        expected: batch.len(),
                        actual: response.items.len(),
                    });
                }

                let mut translated_by_id = std::collections::BTreeMap::new();
                for item in response.items {
                    translated_by_id.insert(item.id, item.text);
                }

                self.report_request();

                for pending in batch {
                    let translated = translated_by_id.remove(&pending.id).ok_or_else(|| {
                        MdTranslatorError::InvalidResponse(format!(
                            "missing translated item for AST node {}",
                            pending.id
                        ))
                    })?;

                    cache_set_if_enabled(
                        &*self.cache,
                        options.cache.enabled,
                        &pending.cache_key,
                        &translated,
                    )
                    .await?;
                    translations.insert(pending.id, translated);
                    summary.translated_lines += 1;
                    self.report_completed(1);
                    self.report_progress();
                }
            }

            return Ok((translations, summary));
        }

        let adaptive = Arc::new(AdaptiveConcurrency::new(&options.runtime));
        let mut remaining = VecDeque::from(batches);

        while !remaining.is_empty() {
            let wave_size = adaptive.current_limit().max(1);
            let mut tasks = Vec::with_capacity(wave_size.min(remaining.len()));

            for _ in 0..wave_size {
                let Some(batch) = remaining.pop_front() else {
                    break;
                };

                let backend = self.backend.clone();
                let cache = self.cache.clone();
                let runtime_cfg = options.runtime.clone();
                let cache_enabled = options.cache.enabled;
                let request = ast_request_from_batch(&batch, options);
                let reporter = self.reporter.clone();
                let provider_name = self.backend.name();
                let adaptive = adaptive.clone();

                tasks.push(tokio::spawn(async move {
                    if let Some(reporter) = &reporter {
                        reporter.state().set_provider(provider_name);
                        reporter.state().record_request();
                    }

                    let response =
                        translate_with_retries(backend, request, &runtime_cfg, adaptive).await?;

                    if response.items.len() != batch.len() {
                        return Err(MdTranslatorError::BatchMismatch {
                            expected: batch.len(),
                            actual: response.items.len(),
                        });
                    }

                    let mut translated_by_id = BTreeMap::new();
                    for item in response.items {
                        translated_by_id.insert(item.id, item.text);
                    }

                    let mut resolved = Vec::with_capacity(batch.len());

                    for pending in batch {
                        let translated = translated_by_id.remove(&pending.id).ok_or_else(|| {
                            MdTranslatorError::InvalidResponse(format!(
                                "missing translated item for AST node {}",
                                pending.id
                            ))
                        })?;

                        cache_set_if_enabled(
                            &*cache,
                            cache_enabled,
                            &pending.cache_key,
                            &translated,
                        )
                        .await?;

                        resolved.push((pending.id, translated));
                    }

                    Ok::<Vec<(usize, String)>, MdTranslatorError>(resolved)
                }));
            }

            let results = join_all(tasks).await;
            for batch_result in results {
                let resolved =
                    batch_result.map_err(|err| MdTranslatorError::Join(err.to_string()))??;

                for (node_id, translated) in resolved {
                    translations.insert(node_id, translated);
                    summary.translated_lines += 1;
                    self.report_completed(1);
                    self.report_progress();
                }
            }
        }

        Ok((translations, summary))
    }

    async fn translate_lines(
        &self,
        mut output_lines: Vec<String>,
        segments: Vec<LineSegment>,
        options: &TranslateOptions,
    ) -> Result<(Vec<String>, TranslationSummary)> {
        let fingerprint = json!({
            "cache_schema": 2,
            "format": match options.format {
                DocumentFormat::Markdown => "markdown",
                DocumentFormat::PlainText => "plain",
            },
            "source": options.source_lang,
            "target": options.target_lang,
            "provider": {
                "name": self.backend.name(),
                "fingerprint": self.backend.cache_fingerprint(),
            },
            "markdown": {
                "translate_frontmatter": options.markdown.translate_frontmatter,
                "translate_multiline_code": options.markdown.translate_multiline_code,
                "translate_latex": options.markdown.translate_latex,
                "translate_link_text": options.markdown.translate_link_text,
            },
            "batching": {
                "max_items_per_batch": options.batching.max_items_per_batch,
                "max_chars_per_batch": options.batching.max_chars_per_batch,
                "context_window": options.batching.context_window,
            },
            "system_prompt": options.system_prompt,
            "user_prompt": options.user_prompt,
        })
        .to_string();

        let mut summary = TranslationSummary {
            total_lines: segments.len(),
            translated_lines: 0,
            cache_hits: 0,
            batches: 0,
            markdown_processing_time_ms: 0,
            provider_time_ms: 0,
        };

        let mut pending = Vec::new();

        self.report_total_items(summary.total_lines);
        self.report_cache_hits(0);
        self.report_completed(0);

        for segment in segments {
            let cache_key = hash_key(&[
                &options.cache.namespace,
                self.backend.name(),
                &fingerprint,
                &segment.text,
            ]);

            if options.cache.enabled
                && let Some(cached) = self.cache.get(&cache_key).await?
            {
                output_lines[segment.line_index] = cached;
                summary.cache_hits += 1;
                summary.translated_lines += 1;
                self.report_cache_hit();
                self.report_completed(1);
                self.report_progress();
                continue;
            }

            pending.push(PendingSegment { segment, cache_key });
        }

        if pending.is_empty() {
            return Ok((output_lines, summary));
        }

        let batches = build_batches(
            &pending,
            options.batching.max_items_per_batch,
            options.batching.max_chars_per_batch,
        );

        summary.batches = batches.len();
        self.report_provider();

        if let Some(multi) =
            (self.backend.as_ref() as &dyn Any).downcast_ref::<MultiProviderBackend>()
        {
            let requests = batches
                .iter()
                .map(|batch| request_from_batch(batch, options))
                .collect::<Vec<_>>();
            let responses = multi
                .translate_document_requests(&requests, &options.runtime)
                .await?;

            for (batch, response) in batches.into_iter().zip(responses) {
                if response.items.len() != batch.len() {
                    return Err(MdTranslatorError::BatchMismatch {
                        expected: batch.len(),
                        actual: response.items.len(),
                    });
                }

                let mut translated_by_id = std::collections::BTreeMap::new();
                for item in response.items {
                    translated_by_id.insert(item.id, item.text);
                }

                self.report_request();

                for pending in batch {
                    let translated = translated_by_id
                        .remove(&pending.segment.line_index)
                        .ok_or_else(|| {
                            MdTranslatorError::InvalidResponse(format!(
                                "missing translated item for line {}",
                                pending.segment.line_index
                            ))
                        })?;

                    cache_set_if_enabled(
                        &*self.cache,
                        options.cache.enabled,
                        &pending.cache_key,
                        &translated,
                    )
                    .await?;
                    output_lines[pending.segment.line_index] = translated;
                    summary.translated_lines += 1;
                    self.report_completed(1);
                    self.report_progress();
                }
            }

            return Ok((output_lines, summary));
        }

        let adaptive = Arc::new(AdaptiveConcurrency::new(&options.runtime));
        let mut remaining = VecDeque::from(batches);

        while !remaining.is_empty() {
            let wave_size = adaptive.current_limit().max(1);
            let mut tasks = Vec::with_capacity(wave_size.min(remaining.len()));

            for _ in 0..wave_size {
                let Some(batch) = remaining.pop_front() else {
                    break;
                };

                let backend = self.backend.clone();
                let cache = self.cache.clone();
                let runtime_cfg = options.runtime.clone();
                let cache_enabled = options.cache.enabled;
                let request = request_from_batch(&batch, options);
                let reporter = self.reporter.clone();
                let provider_name = self.backend.name();
                let adaptive = adaptive.clone();

                tasks.push(tokio::spawn(async move {
                    if let Some(reporter) = &reporter {
                        reporter.state().set_provider(provider_name);
                        reporter.state().record_request();
                    }

                    let response =
                        translate_with_retries(backend, request, &runtime_cfg, adaptive).await?;

                    if response.items.len() != batch.len() {
                        return Err(MdTranslatorError::BatchMismatch {
                            expected: batch.len(),
                            actual: response.items.len(),
                        });
                    }

                    let mut translated_by_id = BTreeMap::new();
                    for item in response.items {
                        translated_by_id.insert(item.id, item.text);
                    }

                    let mut resolved = Vec::with_capacity(batch.len());

                    for pending in batch {
                        let translated = translated_by_id
                            .remove(&pending.segment.line_index)
                            .ok_or_else(|| {
                                MdTranslatorError::InvalidResponse(format!(
                                    "missing translated item for line {}",
                                    pending.segment.line_index
                                ))
                            })?;

                        cache_set_if_enabled(
                            &*cache,
                            cache_enabled,
                            &pending.cache_key,
                            &translated,
                        )
                        .await?;

                        resolved.push((pending.segment.line_index, translated));
                    }

                    Ok::<Vec<(usize, String)>, MdTranslatorError>(resolved)
                }));
            }

            let results = join_all(tasks).await;
            for batch_result in results {
                let resolved =
                    batch_result.map_err(|err| MdTranslatorError::Join(err.to_string()))??;

                for (line_index, translated) in resolved {
                    output_lines[line_index] = translated;
                    summary.translated_lines += 1;
                    self.report_completed(1);
                    self.report_progress();
                }
            }
        }

        Ok((output_lines, summary))
    }

    fn report_provider(&self) {
        if let Some(reporter) = &self.reporter {
            reporter.state().set_provider(self.backend.name());
        }
    }

    fn reset_reporter(&self) {
        if let Some(reporter) = &self.reporter {
            reporter.state().reset();
        }
    }

    fn report_total_items(&self, total: usize) {
        if let Some(reporter) = &self.reporter {
            reporter.state().set_total_items(total);
        }
    }

    fn report_cache_hits(&self, hits: usize) {
        if let Some(reporter) = &self.reporter {
            reporter.state().add_cache_hits(hits);
        }
    }

    fn report_cache_hit(&self) {
        if let Some(reporter) = &self.reporter {
            reporter.state().add_cache_hits(1);
        }
    }

    fn report_request(&self) {
        if let Some(reporter) = &self.reporter {
            reporter.state().record_request();
        }
    }

    fn report_completed(&self, count: usize) {
        if let Some(reporter) = &self.reporter {
            reporter.state().add_completed(count);
        }
    }

    fn report_progress(&self) {
        if let Some(reporter) = &self.reporter {
            reporter.print_progress(
                self.backend.name(),
                reporter.state().completed_items(),
                reporter.state().remaining(),
                reporter.state().cache_hits(),
            );
        }
    }
}

async fn cache_set_if_enabled(
    cache: &dyn Cache,
    enabled: bool,
    key: &str,
    value: &str,
) -> Result<()> {
    if enabled {
        cache.set(key, value).await?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct LineSegment {
    line_index: usize,
    text: String,
    context_before: Vec<String>,
    context_after: Vec<String>,
}

#[derive(Debug, Clone)]
struct PendingSegment {
    segment: LineSegment,
    cache_key: String,
}

type Batch = Vec<PendingSegment>;

#[derive(Debug, Clone)]
struct PendingAstNode {
    id: usize,
    text: String,
    cache_key: String,
}

type AstBatch = Vec<PendingAstNode>;

fn collect_segments(lines: &[String], context_window: usize) -> Vec<LineSegment> {
    let mut segments = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let start = idx.saturating_sub(context_window);
        let end = (idx + context_window + 1).min(lines.len());

        let before = lines[start..idx].to_vec();
        let after = lines[idx + 1..end].to_vec();

        segments.push(LineSegment {
            line_index: idx,
            text: line.clone(),
            context_before: before,
            context_after: after,
        });
    }

    segments
}

fn build_batches(pending: &[PendingSegment], max_items: usize, max_chars: usize) -> Vec<Batch> {
    let max_items = max_items.max(1);
    let max_chars = max_chars.max(1);

    let mut batches = Vec::new();
    let mut current = Vec::new();
    let mut current_chars = 0usize;

    for item in pending.iter().cloned() {
        let item_chars = item.segment.text.chars().count();
        let should_split = !current.is_empty()
            && (current.len() >= max_items || current_chars + item_chars > max_chars);

        if should_split {
            batches.push(std::mem::take(&mut current));
            current_chars = 0;
        }

        current_chars += item_chars;
        current.push(item);
    }

    if !current.is_empty() {
        batches.push(current);
    }

    batches
}

fn build_ast_batches(
    pending: &[PendingAstNode],
    max_items: usize,
    max_chars: usize,
) -> Vec<AstBatch> {
    let max_items = max_items.max(1);
    let max_chars = max_chars.max(1);

    let mut batches = Vec::new();
    let mut current = Vec::new();
    let mut current_chars = 0usize;

    for item in pending.iter().cloned() {
        let item_chars = item.text.chars().count();
        let should_split = !current.is_empty()
            && (current.len() >= max_items || current_chars + item_chars > max_chars);

        if should_split {
            batches.push(std::mem::take(&mut current));
            current_chars = 0;
        }

        current_chars += item_chars;
        current.push(item);
    }

    if !current.is_empty() {
        batches.push(current);
    }

    batches
}

fn request_from_batch(batch: &Batch, options: &TranslateOptions) -> TranslateRequest {
    let items = batch
        .iter()
        .map(|item| BatchItem {
            id: item.segment.line_index,
            text: item.segment.text.clone(),
            context_before: item.segment.context_before.clone(),
            context_after: item.segment.context_after.clone(),
        })
        .collect::<Vec<_>>();

    TranslateRequest {
        source_lang: options.source_lang.clone(),
        target_lang: options.target_lang.clone(),
        items,
        preserve_markdown: matches!(options.format, DocumentFormat::Markdown),
        system_prompt: options.system_prompt.clone(),
        user_prompt: options.user_prompt.clone(),
    }
}

fn ast_request_from_batch(batch: &AstBatch, options: &TranslateOptions) -> TranslateRequest {
    let items = batch
        .iter()
        .map(|item| BatchItem {
            id: item.id,
            text: item.text.clone(),
            context_before: Vec::new(),
            context_after: Vec::new(),
        })
        .collect::<Vec<_>>();

    TranslateRequest {
        source_lang: options.source_lang.clone(),
        target_lang: options.target_lang.clone(),
        items,
        preserve_markdown: true,
        system_prompt: options.system_prompt.clone(),
        user_prompt: options.user_prompt.clone(),
    }
}

async fn translate_with_retries(
    backend: Arc<dyn TranslationBackend>,
    request: TranslateRequest,
    runtime: &RuntimeOptions,
    adaptive: Arc<AdaptiveConcurrency>,
) -> Result<crate::types::TranslateResponse> {
    let mut attempt = 0u32;
    let mut backoff_ms = runtime.retry_backoff_ms.max(1);

    loop {
        let started_at = std::time::Instant::now();
        let result = timeout(
            Duration::from_secs(runtime.request_timeout_secs),
            backend.translate_batch(request.clone()),
        )
        .await;

        let retry_after_hint_ms = retry_after_ms(&result);

        match result {
            Ok(Ok(response)) => {
                adaptive.record_success(started_at.elapsed());
                debug!(
                    backend = backend.name(),
                    attempt,
                    latency_ms = started_at.elapsed().as_millis(),
                    "batch translated"
                );
                return Ok(response);
            }
            Ok(Err(err)) => {
                if !is_retryable_error(&err) {
                    return Err(err);
                }
                adaptive.record_failure(&err);
                if attempt >= runtime.max_retries {
                    return Err(err);
                }
                warn!(backend = backend.name(), attempt, error = %err, "batch translation failed, retrying");
            }
            Err(_) => {
                adaptive.record_timeout();
                if attempt >= runtime.max_retries {
                    return Err(MdTranslatorError::Provider(format!(
                        "request timeout after {} seconds",
                        runtime.request_timeout_secs
                    )));
                }
                warn!(
                    backend = backend.name(),
                    attempt, "batch translation timed out, retrying"
                );
            }
        }

        attempt += 1;
        if let Some(retry_after_ms) = retry_after_hint_ms {
            backoff_ms = backoff_ms.max(retry_after_ms);
        }
        let jitter = rand::rng().random_range(0..=(backoff_ms / 5).max(1));
        tokio::time::sleep(Duration::from_millis(backoff_ms + jitter)).await;
        backoff_ms = (backoff_ms * 2).min(10_000);
    }
}

fn hash_key(parts: &[&str]) -> String {
    let mut hasher = Hasher::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update(&[0]);
    }
    hasher.finalize().to_hex().to_string()
}

fn is_retryable_error(err: &MdTranslatorError) -> bool {
    match err {
        MdTranslatorError::Http(http_err) => {
            http_err.is_timeout()
                || http_err.is_connect()
                || http_err
                    .status()
                    .map(|s| s.as_u16() == 429 || s.is_server_error())
                    .unwrap_or(false)
        }
        MdTranslatorError::Provider(message) => {
            message.contains("status 429")
                || message.contains("status 500")
                || message.contains("status 502")
                || message.contains("status 503")
                || message.contains("status 504")
                || message.to_lowercase().contains("timeout")
        }
        MdTranslatorError::ProviderHttp { status_code, .. } => {
            *status_code == 429 || (500..=504).contains(status_code)
        }
        MdTranslatorError::Io(_) => false,
        MdTranslatorError::Json(_) => false,
        MdTranslatorError::Utf8(_) => false,
        MdTranslatorError::Config(_) => false,
        MdTranslatorError::InvalidResponse(_) => false,
        MdTranslatorError::BatchMismatch { .. } => false,
        MdTranslatorError::Join(_) => false,
    }
}

fn retry_after_ms(
    result: &std::result::Result<
        Result<crate::types::TranslateResponse>,
        tokio::time::error::Elapsed,
    >,
) -> Option<u64> {
    match result {
        Ok(Err(MdTranslatorError::ProviderHttp { retry_after_ms, .. })) => *retry_after_ms,
        _ => None,
    }
}

#[derive(Debug)]
struct AdaptiveConcurrency {
    enabled: bool,
    min_limit: usize,
    max_limit: usize,
    current_limit: AtomicUsize,
    success_streak: AtomicUsize,
    latency_ewma_ms: Mutex<Option<f64>>,
}

impl AdaptiveConcurrency {
    fn new(runtime: &RuntimeOptions) -> Self {
        let max_limit = runtime.max_concurrency.max(1);
        let enabled = runtime.adaptive_concurrency && max_limit > 1;
        let min_limit = if enabled {
            runtime.min_concurrency.clamp(1, max_limit)
        } else {
            max_limit
        };
        let initial_limit = if enabled {
            runtime.initial_concurrency.clamp(min_limit, max_limit)
        } else {
            max_limit
        };

        Self {
            enabled,
            min_limit,
            max_limit,
            current_limit: AtomicUsize::new(initial_limit),
            success_streak: AtomicUsize::new(0),
            latency_ewma_ms: Mutex::new(None),
        }
    }

    fn current_limit(&self) -> usize {
        self.current_limit.load(Ordering::SeqCst).max(1)
    }

    fn record_success(&self, latency: Duration) {
        if !self.enabled {
            return;
        }

        let latency_ms = latency.as_secs_f64() * 1000.0;
        let mut latency_ewma = self
            .latency_ewma_ms
            .lock()
            .expect("adaptive latency mutex poisoned");

        let baseline = match *latency_ewma {
            Some(previous) => {
                let updated = (previous * 0.8) + (latency_ms * 0.2);
                *latency_ewma = Some(updated);
                previous
            }
            None => {
                *latency_ewma = Some(latency_ms);
                latency_ms
            }
        };

        let current = self.current_limit();
        if latency_ms <= baseline * 1.2 {
            let streak = self.success_streak.fetch_add(1, Ordering::SeqCst) + 1;
            if streak >= current && current < self.max_limit {
                self.current_limit.store(current + 1, Ordering::SeqCst);
                self.success_streak.store(0, Ordering::SeqCst);
            }
        } else {
            self.success_streak.store(0, Ordering::SeqCst);
            if latency_ms > baseline * 1.75 {
                self.decrease_soft();
            }
        }
    }

    fn record_failure(&self, err: &MdTranslatorError) {
        if !self.enabled {
            return;
        }

        self.success_streak.store(0, Ordering::SeqCst);
        match err {
            MdTranslatorError::ProviderHttp {
                status_code: 429,
                retry_after_ms,
                ..
            } => self.decrease_hard(*retry_after_ms),
            MdTranslatorError::ProviderHttp { .. } => self.decrease_soft(),
            MdTranslatorError::Provider(message)
                if message.contains("status 429") || message.to_lowercase().contains("timeout") =>
            {
                self.decrease_hard(None)
            }
            _ => self.decrease_soft(),
        }
    }

    fn record_timeout(&self) {
        if !self.enabled {
            return;
        }

        self.success_streak.store(0, Ordering::SeqCst);
        self.decrease_hard(None);
    }

    fn decrease_soft(&self) {
        let current = self.current_limit();
        if current > self.min_limit {
            self.current_limit.store(current - 1, Ordering::SeqCst);
        }
    }

    fn decrease_hard(&self, retry_after_ms: Option<u64>) {
        let current = self.current_limit();
        let halved = (current / 2).max(self.min_limit);
        let reduced = if retry_after_ms.is_some() {
            self.min_limit
        } else {
            halved
        };
        self.current_limit.store(reduced, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod adaptive_tests {
    use super::*;

    #[test]
    fn adaptive_concurrency_grows_after_stable_successes() {
        let runtime = RuntimeOptions {
            max_concurrency: 6,
            adaptive_concurrency: true,
            initial_concurrency: 2,
            min_concurrency: 1,
            max_retries: 3,
            retry_backoff_ms: 300,
            request_timeout_secs: 60,
        };
        let adaptive = AdaptiveConcurrency::new(&runtime);

        adaptive.record_success(Duration::from_millis(100));
        adaptive.record_success(Duration::from_millis(105));

        assert!(adaptive.current_limit() >= 3);
    }

    #[test]
    fn adaptive_concurrency_drops_on_rate_limit() {
        let runtime = RuntimeOptions {
            max_concurrency: 8,
            adaptive_concurrency: true,
            initial_concurrency: 4,
            min_concurrency: 1,
            max_retries: 3,
            retry_backoff_ms: 300,
            request_timeout_secs: 60,
        };
        let adaptive = AdaptiveConcurrency::new(&runtime);
        adaptive.record_failure(&MdTranslatorError::ProviderHttp {
            provider: "openai-compat",
            status_code: 429,
            retry_after_ms: Some(2000),
            message: "rate limited".to_string(),
        });

        assert_eq!(adaptive.current_limit(), 1);
    }
}
