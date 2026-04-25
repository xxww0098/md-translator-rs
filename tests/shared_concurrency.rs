mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use common::{SchedulerProbe, ScriptedOutput, ScriptedWorkUnit, execute_scripted_work_unit};
use md_translator_rs::provider::concurrent::SharedScheduler;

#[tokio::test]
async fn shared_concurrency_bounded_concurrency_enforced() {
    let scheduler = SharedScheduler::new(2);
    let probe = Arc::new(SchedulerProbe::new());
    let units = vec![
        ScriptedWorkUnit {
            id: 0,
            text: "zero".to_string(),
            delay_ms: 40,
            fail_attempts: 0,
        },
        ScriptedWorkUnit {
            id: 1,
            text: "one".to_string(),
            delay_ms: 40,
            fail_attempts: 0,
        },
        ScriptedWorkUnit {
            id: 2,
            text: "two".to_string(),
            delay_ms: 40,
            fail_attempts: 0,
        },
        ScriptedWorkUnit {
            id: 3,
            text: "three".to_string(),
            delay_ms: 40,
            fail_attempts: 0,
        },
    ];

    let probe_for_exec = Arc::clone(&probe);
    let outputs = scheduler
        .execute(units, move |unit| {
            let probe = Arc::clone(&probe_for_exec);
            async move { execute_scripted_work_unit(unit, probe).await }
        })
        .await
        .unwrap();

    assert_eq!(outputs.len(), 4);
    assert_eq!(probe.peak(), 2, "peak concurrency was {}", probe.peak());
    assert_eq!(probe.active(), 0);
    assert_eq!(probe.attempts(), 4);
}

#[tokio::test]
async fn shared_concurrency_stable_ordering_under_out_of_order_completion() {
    let scheduler = SharedScheduler::new(3);
    let probe = Arc::new(SchedulerProbe::new());
    let units = vec![
        ScriptedWorkUnit {
            id: 0,
            text: "zero".to_string(),
            delay_ms: 60,
            fail_attempts: 0,
        },
        ScriptedWorkUnit {
            id: 1,
            text: "one".to_string(),
            delay_ms: 10,
            fail_attempts: 0,
        },
        ScriptedWorkUnit {
            id: 2,
            text: "two".to_string(),
            delay_ms: 20,
            fail_attempts: 0,
        },
    ];

    let probe_for_exec = Arc::clone(&probe);
    let outputs = scheduler
        .execute(units, move |unit| {
            let probe = Arc::clone(&probe_for_exec);
            async move { execute_scripted_work_unit(unit, probe).await }
        })
        .await
        .unwrap();

    let output_ids: Vec<_> = outputs.into_iter().map(|output| output.id).collect();
    assert_eq!(output_ids, vec![0, 1, 2]);
    assert_eq!(probe.attempts(), 3);
}

#[tokio::test]
async fn shared_concurrency_preserves_error_without_internal_retry() {
    let scheduler = SharedScheduler::with_retries(2, 3);
    let probe = Arc::new(SchedulerProbe::new());
    let flaky_attempts = Arc::new(AtomicUsize::new(0));
    let units = vec![
        ScriptedWorkUnit {
            id: 0,
            text: "zero".to_string(),
            delay_ms: 5,
            fail_attempts: 0,
        },
        ScriptedWorkUnit {
            id: 1,
            text: "one".to_string(),
            delay_ms: 5,
            fail_attempts: 0,
        },
        ScriptedWorkUnit {
            id: 2,
            text: "two".to_string(),
            delay_ms: 5,
            fail_attempts: 0,
        },
    ];

    let probe_for_exec = Arc::clone(&probe);
    let flaky_attempts_for_exec = Arc::clone(&flaky_attempts);
    let err = scheduler
        .execute(units, move |unit| {
            let probe = Arc::clone(&probe_for_exec);
            let flaky_attempts = Arc::clone(&flaky_attempts_for_exec);
            async move {
                if unit.id == 1 {
                    probe.start_attempt();
                    tokio::time::sleep(tokio::time::Duration::from_millis(unit.delay_ms)).await;
                    probe.finish_attempt();

                    let attempt = flaky_attempts.fetch_add(1, Ordering::SeqCst);
                    if attempt == 0 {
                        return Err(md_translator_rs::MdTranslatorError::Provider(
                            "unit 1 transient failure".to_string(),
                        ));
                    }

                    return Ok(ScriptedOutput {
                        id: unit.id,
                        text: unit.text,
                    });
                }

                execute_scripted_work_unit(unit, probe).await
            }
        })
        .await
        .expect_err("scheduler should preserve provider retry ownership");

    assert!(err.to_string().contains("unit 1 transient failure"));
    assert_eq!(
        flaky_attempts.load(Ordering::SeqCst),
        1,
        "scheduler should not consume retries that belong to engine/fallback paths"
    );
}
