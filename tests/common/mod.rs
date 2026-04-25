use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use md_translator_rs::{
    BatchItem, MdTranslatorError, Result, TranslateRequest, TranslateResponse,
    TranslateResponseItem, TranslationBackend,
};

pub fn batch_item(id: usize, text: impl Into<String>) -> BatchItem {
    BatchItem {
        id,
        text: text.into(),
        context_before: vec![],
        context_after: vec![],
    }
}

pub fn request_with_items(items: Vec<BatchItem>) -> TranslateRequest {
    TranslateRequest {
        source_lang: "en".to_string(),
        target_lang: "zh".to_string(),
        items,
        preserve_markdown: true,
        system_prompt: None,
        user_prompt: None,
    }
}

#[derive(Debug, Clone)]
pub struct FakeBackendConfig {
    pub name: &'static str,
    pub fingerprint: String,
    pub delay_ms: u64,
    pub reversed: bool,
    pub fail_after: Option<usize>,
}

impl FakeBackendConfig {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            fingerprint: format!("fp-{}", name),
            delay_ms: 0,
            reversed: false,
            fail_after: None,
        }
    }

    pub fn with_delay(mut self, ms: u64) -> Self {
        self.delay_ms = ms;
        self
    }

    pub fn with_reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    pub fn with_fail_after(mut self, n: usize) -> Self {
        self.fail_after = Some(n);
        self
    }
}

#[derive(Debug)]
pub struct FakeBackend {
    config: FakeBackendConfig,
    call_count: AtomicUsize,
}

impl FakeBackend {
    pub fn new(config: FakeBackendConfig) -> Self {
        Self {
            config,
            call_count: AtomicUsize::new(0),
        }
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    fn process_items(&self, items: Vec<BatchItem>) -> Vec<TranslateResponseItem> {
        let mut results: Vec<_> = items
            .into_iter()
            .map(|item| TranslateResponseItem {
                id: item.id,
                text: format!("[{}] {}", self.config.name, item.text),
            })
            .collect();

        if self.config.reversed {
            results.reverse();
        }

        results
    }
}

#[async_trait]
impl TranslationBackend for FakeBackend {
    fn name(&self) -> &'static str {
        self.config.name
    }

    fn cache_fingerprint(&self) -> String {
        self.config.fingerprint.clone()
    }

    async fn translate_batch(&self, request: TranslateRequest) -> Result<TranslateResponse> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);

        if let Some(fail_after) = self.config.fail_after {
            if count >= fail_after {
                return Err(MdTranslatorError::Provider(format!(
                    "{}: simulated failure after {} calls",
                    self.config.name, fail_after
                )));
            }
        }

        if self.config.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.delay_ms)).await;
        }

        let items = self.process_items(request.items);

        Ok(TranslateResponse { items })
    }
}

pub type FakeBackendRef = Arc<FakeBackend>;

pub fn delayed_backend(name: &'static str, delay_ms: u64) -> FakeBackendRef {
    Arc::new(FakeBackend::new(
        FakeBackendConfig::new(name).with_delay(delay_ms),
    ))
}

pub fn reversed_completion_backend(name: &'static str, delay_ms: u64) -> FakeBackendRef {
    Arc::new(FakeBackend::new(
        FakeBackendConfig::new(name)
            .with_delay(delay_ms)
            .with_reversed(),
    ))
}

pub fn failing_backend(name: &'static str, fail_after: usize, delay_ms: u64) -> FakeBackendRef {
    Arc::new(FakeBackend::new(
        FakeBackendConfig::new(name)
            .with_delay(delay_ms)
            .with_fail_after(fail_after),
    ))
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScriptedWorkUnit {
    pub id: usize,
    pub text: String,
    pub delay_ms: u64,
    pub fail_attempts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ScriptedOutput {
    pub id: usize,
    pub text: String,
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct SchedulerProbe {
    active: AtomicUsize,
    peak: AtomicUsize,
    attempts: AtomicUsize,
}

#[allow(dead_code)]
impl SchedulerProbe {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_attempt(&self) {
        self.attempts.fetch_add(1, Ordering::SeqCst);
        let active_now = self.active.fetch_add(1, Ordering::SeqCst) + 1;

        let _ = self
            .peak
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                (active_now > current).then_some(active_now)
            });
    }

    pub fn finish_attempt(&self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn peak(&self) -> usize {
        self.peak.load(Ordering::SeqCst)
    }

    pub fn attempts(&self) -> usize {
        self.attempts.load(Ordering::SeqCst)
    }

    pub fn active(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }
}

#[allow(dead_code)]
pub async fn execute_scripted_work_unit(
    unit: ScriptedWorkUnit,
    probe: Arc<SchedulerProbe>,
) -> Result<ScriptedOutput> {
    probe.start_attempt();

    if unit.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(unit.delay_ms)).await;
    }

    probe.finish_attempt();

    if unit.fail_attempts > 0 {
        return Err(MdTranslatorError::Provider(format!(
            "unit {} failed after retry budget",
            unit.id
        )));
    }

    Ok(ScriptedOutput {
        id: unit.id,
        text: unit.text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_backend_reversed_completion() {
        let backend = reversed_completion_backend("rev-test", 0);

        let items = vec![
            batch_item(0, "first"),
            batch_item(1, "second"),
            batch_item(2, "third"),
        ];

        let response = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();

        assert_eq!(response.items[0].id, 2);
        assert_eq!(response.items[1].id, 1);
        assert_eq!(response.items[2].id, 0);
        assert_eq!(backend.call_count(), 1);
    }

    #[tokio::test]
    async fn fake_backend_failure_simulation() {
        let backend = failing_backend("fail-test", 2, 0);

        let items = vec![batch_item(0, "hello")];
        let response = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();
        assert_eq!(response.items[0].text, "[fail-test] hello");
        assert_eq!(backend.call_count(), 1);

        let items = vec![batch_item(1, "world")];
        backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();
        assert_eq!(backend.call_count(), 2);

        let items = vec![batch_item(2, "fail")];
        let err = backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("simulated failure"));
        assert_eq!(backend.call_count(), 3);
    }

    #[tokio::test]
    async fn fake_backend_delay() {
        let backend = delayed_backend("delay-test", 10);

        let start = std::time::Instant::now();
        let items = vec![batch_item(0, "hello")];
        backend
            .translate_batch(request_with_items(items))
            .await
            .unwrap();
        let elapsed = start.elapsed().as_millis() as u64;

        assert!(elapsed >= 9, "expected >=9ms, got {}ms", elapsed);
    }
}
