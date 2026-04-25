//! TranslationReporter - tracks and reports translation progress
//!
//! Tracks: requests sent, cache hits, provider used, remaining items.
//! Prints progress to stderr during translation and final summary after completion.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Summary data passed to [`TranslationReporter::print_summary`].
#[derive(Debug, Clone)]
pub struct SummaryReport<'a> {
    pub provider_name: &'a str,
    pub total_lines: usize,
    pub translated_lines: usize,
    pub cache_hits: usize,
    pub batches: usize,
    pub markdown_processing_time_ms: u64,
    pub provider_time_ms: u64,
}

/// Reporter state shared between CLI and translation pipeline
#[derive(Debug, Clone)]
pub struct ReporterState {
    /// Provider name used for translation
    provider_name: Arc<AtomicUsize>,
    /// Total requests/batches sent
    requests_sent: Arc<AtomicUsize>,
    /// Cache hits during translation
    cache_hits: Arc<AtomicUsize>,
    /// Total items (lines/segments) to translate
    total_items: Arc<AtomicUsize>,
    /// Items completed so far
    completed_items: Arc<AtomicUsize>,
}

impl ReporterState {
    /// Create a new reporter state
    pub fn new() -> Self {
        Self {
            provider_name: Arc::new(AtomicUsize::new(0)),
            requests_sent: Arc::new(AtomicUsize::new(0)),
            cache_hits: Arc::new(AtomicUsize::new(0)),
            total_items: Arc::new(AtomicUsize::new(0)),
            completed_items: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Set the provider name (stored as pointer address for identity)
    pub fn set_provider(&self, name: &str) {
        // Use pointer address as a simple identity token
        let ptr = name.as_ptr() as usize;
        self.provider_name.store(ptr, Ordering::SeqCst);
    }

    /// Record that a request was sent
    pub fn record_request(&self) {
        self.requests_sent.fetch_add(1, Ordering::SeqCst);
    }

    /// Record cache hits
    pub fn add_cache_hits(&self, hits: usize) {
        self.cache_hits.fetch_add(hits, Ordering::SeqCst);
    }

    /// Set total items to translate
    pub fn set_total_items(&self, total: usize) {
        self.total_items.store(total, Ordering::SeqCst);
    }

    /// Reset counters for a new translation run
    pub fn reset(&self) {
        self.provider_name.store(0, Ordering::SeqCst);
        self.requests_sent.store(0, Ordering::SeqCst);
        self.cache_hits.store(0, Ordering::SeqCst);
        self.total_items.store(0, Ordering::SeqCst);
        self.completed_items.store(0, Ordering::SeqCst);
    }

    /// Record completed items
    pub fn add_completed(&self, count: usize) {
        self.completed_items.fetch_add(count, Ordering::SeqCst);
    }

    /// Get remaining items
    pub fn remaining(&self) -> usize {
        let total = self.total_items.load(Ordering::SeqCst);
        let completed = self.completed_items.load(Ordering::SeqCst);
        total.saturating_sub(completed)
    }

    /// Get provider name from stored pointer (for display purposes)
    pub fn get_provider_ptr(&self) -> usize {
        self.provider_name.load(Ordering::SeqCst)
    }

    /// Get requests sent count
    pub fn requests_sent(&self) -> usize {
        self.requests_sent.load(Ordering::SeqCst)
    }

    /// Get cache hits count
    pub fn cache_hits(&self) -> usize {
        self.cache_hits.load(Ordering::SeqCst)
    }

    /// Get total items
    pub fn total_items(&self) -> usize {
        self.total_items.load(Ordering::SeqCst)
    }

    /// Get completed items
    pub fn completed_items(&self) -> usize {
        self.completed_items.load(Ordering::SeqCst)
    }
}

impl Default for ReporterState {
    fn default() -> Self {
        Self::new()
    }
}

/// Verbosity level for reporter output
#[derive(Debug, Clone, Copy, Default)]
pub enum Verbosity {
    /// Quiet mode - suppress all non-error output
    Quiet,
    /// Normal mode - standard summary output
    #[default]
    Normal,
    /// Verbose mode - detailed progress output
    Verbose,
}

/// TranslationReporter - formats and emits translation progress reports
#[derive(Debug, Clone)]
pub struct TranslationReporter {
    state: ReporterState,
    verbosity: Verbosity,
    concurrency: usize,
}

impl TranslationReporter {
    /// Create a new translation reporter
    pub fn new(verbosity: Verbosity, concurrency: usize) -> Self {
        Self {
            state: ReporterState::new(),
            verbosity,
            concurrency,
        }
    }

    /// Get the reporter state for wiring into translation pipeline
    pub fn state(&self) -> &ReporterState {
        &self.state
    }

    /// Get current verbosity
    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }

    /// Get configured concurrency
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    /// Print final summary to stderr
    pub fn print_summary(&self, report: &SummaryReport<'_>) {
        match self.verbosity {
            Verbosity::Quiet => return,
            Verbosity::Normal | Verbosity::Verbose => {}
        }

        eprintln!("--- Translation Summary ---");
        eprintln!("Provider: {}", report.provider_name);
        eprintln!("Total lines: {}", report.total_lines);
        eprintln!("Translated lines: {}", report.translated_lines);
        eprintln!("Cache hits: {}", report.cache_hits);
        eprintln!("Batches (requests): {}", report.batches);
        eprintln!(
            "Rust markdown prep time: {:.2}s",
            report.markdown_processing_time_ms as f64 / 1000.0
        );
        eprintln!(
            "External translation time: {:.2}s",
            report.provider_time_ms as f64 / 1000.0
        );
        eprintln!("Concurrency: {}", self.concurrency);
        eprintln!("--------------------------");
    }

    /// Print progress update to stderr (verbose mode only)
    pub fn print_progress(
        &self,
        provider_name: &str,
        completed: usize,
        remaining: usize,
        cache_hits: usize,
    ) {
        if !matches!(self.verbosity, Verbosity::Verbose) {
            return;
        }

        eprintln!(
            "[{}] Progress: {} done, {} remaining, {} cache hits",
            provider_name, completed, remaining, cache_hits
        );
    }

    /// Check if quiet mode is enabled
    pub fn is_quiet(&self) -> bool {
        matches!(self.verbosity, Verbosity::Quiet)
    }
}

impl Default for TranslationReporter {
    fn default() -> Self {
        Self::new(Verbosity::default(), 6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reporter_state_tracking() {
        let state = ReporterState::new();

        state.set_total_items(100);
        assert_eq!(state.total_items(), 100);

        state.add_completed(30);
        assert_eq!(state.completed_items(), 30);
        assert_eq!(state.remaining(), 70);

        state.add_cache_hits(10);
        assert_eq!(state.cache_hits(), 10);

        state.record_request();
        state.record_request();
        assert_eq!(state.requests_sent(), 2);
    }

    #[test]
    fn test_reporter_quiet_mode() {
        let reporter = TranslationReporter::new(Verbosity::Quiet, 4);
        assert!(reporter.is_quiet());
    }

    #[test]
    fn test_reporter_verbose_mode() {
        let reporter = TranslationReporter::new(Verbosity::Verbose, 4);
        assert!(!reporter.is_quiet());
    }

    #[test]
    fn test_reporter_output() {
        let reporter = TranslationReporter::new(Verbosity::Normal, 6);
        assert_eq!(reporter.concurrency(), 6);
        assert!(!reporter.is_quiet());

        reporter.print_summary(&SummaryReport {
            provider_name: "gtx",
            total_lines: 100,
            translated_lines: 80,
            cache_hits: 15,
            batches: 5,
            markdown_processing_time_ms: 100,
            provider_time_ms: 1000,
        });

        assert_eq!(reporter.state().total_items(), 0);
        assert_eq!(reporter.state().cache_hits(), 0);
    }

    #[test]
    fn test_reporter_quiet_suppresses_summary() {
        let reporter = TranslationReporter::new(Verbosity::Quiet, 4);
        assert!(reporter.is_quiet());
        reporter.print_summary(&SummaryReport {
            provider_name: "gtx",
            total_lines: 100,
            translated_lines: 80,
            cache_hits: 15,
            batches: 5,
            markdown_processing_time_ms: 100,
            provider_time_ms: 1000,
        });
    }

    #[test]
    fn test_reporter_verbose_mode_progress() {
        let reporter = TranslationReporter::new(Verbosity::Verbose, 6);
        assert!(!reporter.is_quiet());
        reporter.print_progress("gtx", 50, 50, 10);
    }

    #[test]
    fn test_reporter_state_concurrency() {
        let reporter = TranslationReporter::new(Verbosity::Normal, 8);
        assert_eq!(reporter.concurrency(), 8);
    }
}
