use std::future::Future;

use async_trait::async_trait;
use futures::stream::{self, StreamExt};

use crate::error::Result;
use crate::types::{TranslateRequest, TranslateResponse};

/// Provider-side adapter contract that preserves each backend's natural work-unit
/// split while allowing shared scheduler execution.
///
/// Four concrete work-unit styles are supported via associated types:
/// - **Item unit** — one HTTP request per `BatchItem` (GTX, DeepLX).
/// - **Chunk unit** — one HTTP request per bounded slice of `BatchItem`s
///   (DeepL native batching, up to 50 texts / 130 KB per chunk).
/// - **XML-batch unit** — one HTTP request per XML-wrapped batch of segments
///   (OpenAI-compatible, default 15 segments per batch).
#[async_trait]
pub(crate) trait ConcurrentTranslationBackend {
    type WorkUnit: Send + Clone;
    type WorkOutput: Send;

    fn split_work(&self, request: &TranslateRequest) -> Vec<Self::WorkUnit>;

    async fn execute_work_unit(&self, unit: Self::WorkUnit) -> Result<Self::WorkOutput>;

    async fn assemble_response(
        &self,
        request: &TranslateRequest,
        outputs: Vec<Self::WorkOutput>,
    ) -> Result<TranslateResponse>;

    /// Execute the full request through the shared scheduler using this
    /// backend's work-unit split and bounded concurrency.
    ///
    /// This is the adapter hook that migration tasks (T10–T14) call
    /// instead of hand-rolling `for` loops, `try_join_all`, or local
    /// semaphore gating inside each provider.
    async fn execute_concurrently(
        &self,
        request: &TranslateRequest,
        max_concurrency: usize,
    ) -> Result<TranslateResponse>
    where
        Self: Sync,
    {
        let work = self.split_work(request);
        let scheduler = SharedScheduler::new(max_concurrency);
        let outputs = scheduler
            .execute(work, |unit| self.execute_work_unit(unit))
            .await?;
        self.assemble_response(request, outputs).await
    }
}

pub struct SharedScheduler {
    limit: usize,
}

impl SharedScheduler {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }

    pub fn with_retries(limit: usize, _max_retries: usize) -> Self {
        Self::new(limit)
    }

    pub async fn execute<W, O, F, Fut>(&self, units: Vec<W>, execute: F) -> Result<Vec<O>>
    where
        W: Send + Clone,
        O: Send,
        F: Fn(W) -> Fut + Send + Sync,
        Fut: Future<Output = Result<O>> + Send,
    {
        let effective_limit = self.limit.max(1);
        let mut indexed_results = stream::iter(units.into_iter().enumerate())
            .map(|(idx, unit)| {
                let execute = &execute;
                async move {
                    let result = execute(unit).await;
                    (idx, result)
                }
            })
            .buffer_unordered(effective_limit)
            .collect::<Vec<_>>()
            .await;

        indexed_results.sort_by_key(|(idx, _)| *idx);

        let mut outputs = Vec::with_capacity(indexed_results.len());
        for (_, result) in indexed_results {
            outputs.push(result?);
        }

        Ok(outputs)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use tokio::time::sleep;

    use super::*;
    use crate::error::MdTranslatorError;

    #[derive(Clone)]
    struct TestUnit {
        id: usize,
        delay_ms: u64,
        fail: bool,
    }

    #[derive(Default)]
    struct Probe {
        active: AtomicUsize,
        peak: AtomicUsize,
        attempts: AtomicUsize,
    }

    #[derive(Default)]
    struct AttemptRegistry {
        attempts: Mutex<Vec<usize>>,
    }

    impl AttemptRegistry {
        fn record(&self, id: usize) {
            self.attempts
                .lock()
                .expect("attempt registry mutex poisoned")
                .push(id);
        }

        fn snapshot(&self) -> Vec<usize> {
            self.attempts
                .lock()
                .expect("attempt registry mutex poisoned")
                .clone()
        }
    }

    impl Probe {
        fn start(&self) {
            self.attempts.fetch_add(1, Ordering::SeqCst);
            let active_now = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            let _ = self
                .peak
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                    (active_now > current).then_some(active_now)
                });
        }

        fn finish(&self) {
            self.active.fetch_sub(1, Ordering::SeqCst);
        }

        fn peak(&self) -> usize {
            self.peak.load(Ordering::SeqCst)
        }

        fn attempts(&self) -> usize {
            self.attempts.load(Ordering::SeqCst)
        }

        fn active(&self) -> usize {
            self.active.load(Ordering::SeqCst)
        }
    }

    async fn execute_test_unit(unit: TestUnit, probe: Arc<Probe>) -> Result<usize> {
        probe.start();

        if unit.delay_ms > 0 {
            sleep(Duration::from_millis(unit.delay_ms)).await;
        }

        probe.finish();

        if unit.fail {
            return Err(MdTranslatorError::Provider(format!(
                "unit {} failed",
                unit.id
            )));
        }

        Ok(unit.id)
    }

    async fn execute_test_unit_with_registry(
        unit: TestUnit,
        probe: Arc<Probe>,
        attempts: Arc<AttemptRegistry>,
    ) -> Result<usize> {
        attempts.record(unit.id);
        execute_test_unit(unit, probe).await
    }

    #[tokio::test]
    async fn reversed_completion_preserves_input_order() {
        let scheduler = SharedScheduler::new(3);
        let probe = Arc::new(Probe::default());
        let units = vec![
            TestUnit {
                id: 0,
                delay_ms: 60,
                fail: false,
            },
            TestUnit {
                id: 1,
                delay_ms: 20,
                fail: false,
            },
            TestUnit {
                id: 2,
                delay_ms: 5,
                fail: false,
            },
        ];

        let probe_for_exec = Arc::clone(&probe);
        let outputs = scheduler
            .execute(units, move |unit| {
                let probe = Arc::clone(&probe_for_exec);
                async move { execute_test_unit(unit, probe).await }
            })
            .await
            .unwrap();

        assert_eq!(outputs, vec![0, 1, 2]);
        assert_eq!(probe.attempts(), 3);
    }

    #[tokio::test]
    async fn concurrency_bound_caps_in_flight_work() {
        let scheduler = SharedScheduler::new(2);
        let probe = Arc::new(Probe::default());
        let units = vec![
            TestUnit {
                id: 0,
                delay_ms: 40,
                fail: false,
            },
            TestUnit {
                id: 1,
                delay_ms: 40,
                fail: false,
            },
            TestUnit {
                id: 2,
                delay_ms: 40,
                fail: false,
            },
            TestUnit {
                id: 3,
                delay_ms: 40,
                fail: false,
            },
        ];

        let probe_for_exec = Arc::clone(&probe);
        let outputs = scheduler
            .execute(units, move |unit| {
                let probe = Arc::clone(&probe_for_exec);
                async move { execute_test_unit(unit, probe).await }
            })
            .await
            .unwrap();

        assert_eq!(outputs, vec![0, 1, 2, 3]);
        assert_eq!(probe.peak(), 2);
        assert_eq!(probe.active(), 0);
        assert_eq!(probe.attempts(), 4);
    }

    #[tokio::test]
    async fn scheduler_propagates_lowest_index_error_after_out_of_order_completion() {
        let scheduler = SharedScheduler::with_retries(2, 0);
        let probe = Arc::new(Probe::default());
        let units = vec![
            TestUnit {
                id: 0,
                delay_ms: 40,
                fail: true,
            },
            TestUnit {
                id: 1,
                delay_ms: 5,
                fail: true,
            },
        ];

        let probe_for_exec = Arc::clone(&probe);
        let err = scheduler
            .execute(units, move |unit| {
                let probe = Arc::clone(&probe_for_exec);
                async move { execute_test_unit(unit, probe).await }
            })
            .await
            .unwrap_err();

        assert!(err.to_string().contains("unit 0 failed"));
        assert_eq!(probe.peak(), 2);
        assert_eq!(probe.active(), 0);
        assert_eq!(probe.attempts(), 2);
    }

    #[tokio::test]
    async fn scheduler_does_not_retry_work_units_internally() {
        let scheduler = SharedScheduler::with_retries(2, 3);
        let probe = Arc::new(Probe::default());
        let attempts = Arc::new(AttemptRegistry::default());
        let units = vec![
            TestUnit {
                id: 0,
                delay_ms: 5,
                fail: false,
            },
            TestUnit {
                id: 1,
                delay_ms: 5,
                fail: true,
            },
        ];

        let probe_for_exec = Arc::clone(&probe);
        let attempts_for_exec = Arc::clone(&attempts);
        let err = scheduler
            .execute(units, move |unit| {
                let probe = Arc::clone(&probe_for_exec);
                let attempts = Arc::clone(&attempts_for_exec);
                async move { execute_test_unit_with_registry(unit, probe, attempts).await }
            })
            .await
            .unwrap_err();

        assert!(err.to_string().contains("unit 1 failed"));
        assert_eq!(attempts.snapshot(), vec![0, 1]);
        assert_eq!(probe.attempts(), 2);
    }

    #[tokio::test]
    async fn scheduler_preserves_provider_http_retry_metadata() {
        let scheduler = SharedScheduler::with_retries(2, 5);
        let err = scheduler
            .execute(vec![0usize], |_unit| async move {
                Err::<usize, MdTranslatorError>(MdTranslatorError::ProviderHttp {
                    provider: "deeplx",
                    status_code: 429,
                    retry_after_ms: Some(1_500),
                    message: "rate limited".to_string(),
                })
            })
            .await
            .unwrap_err();

        match err {
            MdTranslatorError::ProviderHttp {
                provider,
                status_code,
                retry_after_ms,
                message,
            } => {
                assert_eq!(provider, "deeplx");
                assert_eq!(status_code, 429);
                assert_eq!(retry_after_ms, Some(1_500));
                assert_eq!(message, "rate limited");
            }
            other => panic!("expected ProviderHttp error, got {other:?}"),
        }
    }
}
