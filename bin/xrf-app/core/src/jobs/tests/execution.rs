use std::future::Future;
use std::num::NonZeroUsize;
use std::sync::{Arc, mpsc};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use serde::Serialize;
use serde_json::json;
use uuid::Uuid;
use xrf_job::{ExecutionRequest, JobOutcome};

use crate::execution::ExecutionState;
use crate::jobs::job_conclusion::JobConclusion;
use crate::jobs::tests::kinds;
use crate::jobs::{JobRegistry, JobResource, JobStart};

#[derive(Debug, Serialize)]
struct ResultSummary {
  outcome: JobOutcome,
  count: u32,
}

fn new_execution() -> ExecutionState {
  ExecutionState::new(ExecutionRequest::Workers(NonZeroUsize::new(1).expect("one worker"))).expect("pool starts")
}

fn new_start() -> JobStart {
  JobStart::new(Uuid::new_v4(), kinds::ARCHIVES_PACK)
    .with_exclusion_group("test")
    .with_resources(vec![JobResource::file(
      std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output"),
    )])
}

#[test]
fn result_classifies_completion_even_after_a_cancel_request() {
  for outcome in [JobOutcome::Completed, JobOutcome::Cancelled] {
    let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
    let execution: ExecutionState = new_execution();

    let result = tauri::async_runtime::block_on(registry.run(
      &execution,
      new_start(),
      move |job| {
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
  let execution: ExecutionState = new_execution();

  let result = tauri::async_runtime::block_on(registry.run(
    &execution,
    new_start(),
    |job| {
      job.cancel();

      Err::<ResultSummary, _>("cannot write")
    },
    |summary| summary.outcome,
  ));

  assert_eq!(result.unwrap_err(), "cannot write");
  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Failed));
  assert_eq!(registry.list()[0].error.as_deref(), Some("cannot write"));

  registry.register(new_start()).expect("leases released");
}

#[test]
fn a_start_the_registry_refuses_runs_nothing() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = new_execution();
  let (_job, _registration) = registry.register(new_start()).expect("first job registers");
  let (ran, wait_ran) = mpsc::channel::<()>();

  let result = tauri::async_runtime::block_on(registry.run(
    &execution,
    new_start(),
    move |_| {
      ran.send(()).expect("ran signal");

      Ok::<_, String>(ResultSummary {
        outcome: JobOutcome::Completed,
        count: 0,
      })
    },
    |summary| summary.outcome,
  ));

  assert!(result.is_err());
  assert!(wait_ran.try_recv().is_err());
}

#[test]
fn dropping_the_awaiting_future_keeps_running_work_registered() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = new_execution();
  let start: JobStart = new_start();
  let id: Uuid = start.get_id();

  let (started, wait_started) = mpsc::channel();
  let (finish, wait_finish) = mpsc::channel();

  let mut future = Box::pin(registry.run(
    &execution,
    start,
    move |_| {
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
      .register(JobStart::new(Uuid::new_v4(), kinds::ARCHIVES_PACK).with_exclusion_group("test"))
      .is_err()
  );
  assert!(
    registry
      .register(
        JobStart::new(Uuid::new_v4(), kinds::ARCHIVES_UNPACK).with_resources(vec![JobResource::file(
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

  registry.register(new_start()).expect("leases released");
}

#[test]
fn dropping_a_queued_job_future_keeps_registration_until_the_worker_runs() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = new_execution();

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

  let (finished, wait_finished) = mpsc::channel();

  let mut queued = Box::pin(registry.run(
    &execution,
    new_start(),
    move |_| {
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
      .register(JobStart::new(Uuid::new_v4(), kinds::ARCHIVES_PACK).with_exclusion_group("test"))
      .is_err()
  );

  release.send(()).expect("release pool");

  tauri::async_runtime::block_on(blocker).expect("blocker ends");

  wait_finished
    .recv_timeout(Duration::from_secs(5))
    .expect("queued work finishes");

  tauri::async_runtime::block_on(execution.run_blocking("drain", || ())).expect("pool drained");

  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Completed));

  registry.register(new_start()).expect("leases released");
}

#[test]
fn a_panicking_worker_releases_registration_as_failed() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let execution: ExecutionState = new_execution();

  let result = tauri::async_runtime::block_on(registry.run(
    &execution,
    new_start(),
    |_| -> Result<ResultSummary, String> { panic!("worker panicked") },
    |summary| summary.outcome,
  ));

  assert!(result.unwrap_err().contains("Job archives.pack did not finish"));
  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Failed));

  registry.register(new_start()).expect("leases released");
}
