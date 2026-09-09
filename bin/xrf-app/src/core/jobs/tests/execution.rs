use std::future::Future;
use std::num::NonZeroUsize;
use std::sync::{Arc, mpsc};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use xrf_job::{ExecutionRequest, JobHandle, JobOutcome};

use crate::core::execution::ExecutionState;
use crate::core::jobs::job_conclusion::JobConclusion;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};

#[derive(Debug, Serialize)]
struct ResultSummary {
  outcome: JobOutcome,
  count: u32,
}

fn execution() -> ExecutionState {
  ExecutionState::new(ExecutionRequest::Workers(NonZeroUsize::new(1).expect("one worker"))).expect("pool starts")
}

fn register(registry: &Arc<JobRegistry>) -> (JobHandle, JobRegistration) {
  registry
    .register(
      JobStart::new(Uuid::new_v4(), JobKind::ArchivesPack)
        .with_exclusion_group("test")
        .with_resources(vec![JobResource::file(
          std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output"),
        )]),
    )
    .expect("job registers")
}

#[test]
fn result_classifies_completion_even_after_a_cancel_request() {
  for outcome in [JobOutcome::Completed, JobOutcome::Cancelled] {
    let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
    let execution: ExecutionState = execution();
    let (job, registration): (JobHandle, JobRegistration) = register(&registry);

    let result = tauri::async_runtime::block_on(run_job(
      &execution,
      "test",
      registration,
      move || {
        job.cancel();

        Ok::<_, String>(ResultSummary { outcome, count: 3 })
      },
      |summary| summary.outcome,
    ))
    .expect("reported result");

    assert_eq!(result.outcome, outcome);
    let listed = registry.list();
    let expected: JobConclusion = match outcome {
      JobOutcome::Completed => JobConclusion::Completed,
      JobOutcome::Cancelled => JobConclusion::Cancelled,
    };

    assert_eq!(listed[0].conclusion, Some(expected));
    assert_eq!(listed[0].result, Some(serde_json::to_value(result).expect("summary")));
  }
}

#[test]
fn a_worker_error_is_retained_and_releases_the_leases() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = execution();
  let (job, registration): (JobHandle, JobRegistration) = register(&registry);

  job.cancel();

  let result = tauri::async_runtime::block_on(run_job(
    &execution,
    "test",
    registration,
    || Err::<ResultSummary, _>("cannot write"),
    |summary| summary.outcome,
  ));

  assert_eq!(result.unwrap_err(), "cannot write");
  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Failed));
  assert_eq!(registry.list()[0].error.as_deref(), Some("cannot write"));
  register(&registry);
}

#[test]
fn dropping_the_awaiting_future_keeps_running_work_registered() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = execution();
  let (_job, registration): (JobHandle, JobRegistration) = register(&registry);
  let id: Uuid = registration.id();
  let (started, wait_started) = mpsc::channel();
  let (finish, wait_finish) = mpsc::channel();
  let mut future = Box::pin(run_job(
    &execution,
    "test",
    registration,
    move || {
      started.send(()).expect("started signal");
      wait_finish.recv_timeout(Duration::from_secs(5)).expect("finish signal");

      Ok::<_, String>(ResultSummary {
        outcome: JobOutcome::Completed,
        count: 7,
      })
    },
    |summary| summary.outcome,
  ));

  assert!(matches!(
    future.as_mut().poll(&mut Context::from_waker(Waker::noop())),
    Poll::Pending
  ));
  wait_started
    .recv_timeout(Duration::from_secs(5))
    .expect("worker started");
  drop(future);
  registry.cancel(id);

  let listed = registry.list();

  assert_eq!(listed[0].conclusion, None);
  assert!(listed[0].is_cancel_requested);
  assert!(
    registry
      .register(JobStart::new(Uuid::new_v4(), JobKind::ArchivesPack).with_exclusion_group("test"))
      .is_err()
  );
  assert!(
    registry
      .register(
        JobStart::new(Uuid::new_v4(), JobKind::ArchivesUnpack).with_resources(vec![JobResource::file(
          std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output")
        )])
      )
      .is_err()
  );

  finish.send(()).expect("finish worker");
  // One worker: this cannot run until the blocked job has recorded its result and released registration.
  tauri::async_runtime::block_on(execution.run_blocking("drain", || ())).expect("pool drained");

  let listed = registry.list();

  assert_eq!(listed[0].conclusion, Some(JobConclusion::Completed));
  assert_eq!(listed[0].result, Some(json!({ "outcome": "completed", "count": 7 })));
  register(&registry);
}

#[test]
fn dropping_a_queued_job_future_keeps_registration_until_the_worker_runs() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = execution();
  let (occupied, wait_occupied) = mpsc::channel();
  let (release, wait_release) = mpsc::channel();
  let mut blocker = Box::pin(execution.run_blocking("occupy the pool", move || {
    occupied.send(()).expect("pool occupied");
    wait_release.recv_timeout(Duration::from_secs(5)).expect("release pool");
  }));

  assert!(matches!(
    blocker.as_mut().poll(&mut Context::from_waker(Waker::noop())),
    Poll::Pending
  ));
  wait_occupied
    .recv_timeout(Duration::from_secs(5))
    .expect("worker occupied");

  let (_job, registration): (JobHandle, JobRegistration) = register(&registry);
  let (finished, wait_finished) = mpsc::channel();
  let mut queued = Box::pin(run_job(
    &execution,
    "queued job",
    registration,
    move || {
      finished.send(()).expect("work ran");

      Ok::<_, String>(ResultSummary {
        outcome: JobOutcome::Completed,
        count: 1,
      })
    },
    |summary| summary.outcome,
  ));

  assert!(matches!(
    queued.as_mut().poll(&mut Context::from_waker(Waker::noop())),
    Poll::Pending
  ));
  drop(queued);

  assert!(wait_finished.try_recv().is_err());
  assert_eq!(registry.list()[0].conclusion, None);
  assert!(
    registry
      .register(JobStart::new(Uuid::new_v4(), JobKind::ArchivesPack).with_exclusion_group("test"))
      .is_err()
  );

  release.send(()).expect("release pool");
  tauri::async_runtime::block_on(blocker).expect("blocker ends");
  wait_finished
    .recv_timeout(Duration::from_secs(5))
    .expect("queued work finishes");
  tauri::async_runtime::block_on(execution.run_blocking("drain", || ())).expect("pool drained");

  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Completed));
  register(&registry);
}

#[test]
fn a_panicking_worker_releases_registration_as_failed() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = execution();
  let (_job, registration): (JobHandle, JobRegistration) = register(&registry);
  let result = tauri::async_runtime::block_on(run_job(
    &execution,
    "test",
    registration,
    || -> Result<ResultSummary, String> { panic!("worker panicked") },
    |summary| summary.outcome,
  ));

  assert!(result.unwrap_err().contains("test did not finish"));
  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Failed));
  register(&registry);
}
