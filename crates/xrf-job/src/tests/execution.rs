use std::num::NonZeroUsize;
use std::str::FromStr;
use std::thread;

use crate::execution_origin::ExecutionOrigin;
use crate::execution_plan::ExecutionPlan;
use crate::execution_request::ExecutionRequest;

fn read(text: &str) -> ExecutionRequest {
  ExecutionRequest::from_str(text).expect("the request is readable")
}

/// The message a rejected request answers with, which is the part a person retypes from.
fn rejection(text: &str) -> String {
  ExecutionRequest::from_str(text)
    .expect_err("the request is rejected")
    .to_string()
}

fn workers(count: usize) -> NonZeroUsize {
  NonZeroUsize::new(count).expect("test counts are not zero")
}

fn host() -> NonZeroUsize {
  thread::available_parallelism().unwrap_or(NonZeroUsize::MIN)
}

#[test]
fn reads_auto_whatever_case_and_spacing_it_arrives_in() {
  assert_eq!(read("auto"), ExecutionRequest::Auto);
  assert_eq!(read("AUTO"), ExecutionRequest::Auto);
  assert_eq!(read("  Auto  "), ExecutionRequest::Auto);
}

#[test]
fn reads_a_worker_count() {
  assert_eq!(read("1"), ExecutionRequest::Workers(workers(1)));
  assert_eq!(read("12"), ExecutionRequest::Workers(workers(12)));
}

#[test]
fn reads_a_share_of_the_host() {
  assert_eq!(read("50%"), ExecutionRequest::Share(workers(50)));
  assert_eq!(read("100%"), ExecutionRequest::Share(workers(100)));
  assert_eq!(read("25 %"), ExecutionRequest::Share(workers(25)));
}

#[test]
fn rejects_a_plan_with_no_workers_in_it() {
  assert!(rejection("0").contains("at least 1"));
  assert!(rejection("0%").contains("at least 1%"));
}

#[test]
fn accepts_the_ceiling_and_rejects_the_typo_above_it() {
  assert_eq!(
    read(&ExecutionRequest::MAX_WORKERS.to_string()),
    ExecutionRequest::Workers(workers(ExecutionRequest::MAX_WORKERS))
  );

  // The message names the limit, because a rejected command line is retyped rather than looked up.
  let rejected: String = rejection(&(ExecutionRequest::MAX_WORKERS + 1).to_string());

  assert!(rejected.contains(&ExecutionRequest::MAX_WORKERS.to_string()));
  assert!(rejected.contains("at most"));
}

#[test]
fn rejects_a_share_larger_than_the_machine() {
  // A share is a fraction of the host. Oversubscription is available by naming the count outright.
  assert!(rejection("150%").contains("at most 100%"));
}

#[test]
fn rejects_what_it_cannot_read_and_says_what_it_accepts() {
  for text in ["", "  ", "sequential", "half", "-4", "4.5", "%", "1e3", "4 workers"] {
    let rejected: String = rejection(text);

    assert!(rejected.contains("auto"), "{text} was rejected without naming `auto`");
    assert!(rejected.contains("50%"), "{text} was rejected without naming a share");
  }
}

#[test]
fn resolves_auto_to_the_host_without_claiming_anybody_chose_it() {
  let plan: ExecutionPlan = ExecutionRequest::Auto.resolve();

  assert_eq!(plan.get_workers(), host());
  assert_eq!(plan.get_origin(), ExecutionOrigin::Auto);
}

#[test]
fn resolves_a_named_count_to_itself_and_records_that_it_was_named() {
  let plan: ExecutionPlan = read("3").resolve();

  assert_eq!(plan.get_workers(), workers(3));
  // The distinction an operation needs before it may decide to use less than it was given.
  assert_eq!(plan.get_origin(), ExecutionOrigin::Requested);
}

#[test]
fn resolves_a_full_share_to_the_whole_host() {
  let plan: ExecutionPlan = read("100%").resolve();

  assert_eq!(plan.get_workers(), host());
  assert_eq!(plan.get_origin(), ExecutionOrigin::Requested);
}

#[test]
fn rounds_a_share_down_but_never_away() {
  // Down rather than to nearest, so a share is a promise not to exceed a fraction of the machine.
  assert_eq!(
    read("50%").resolve().get_workers().get(),
    (host().get() / 2).max(1),
    "half of {} workers",
    host()
  );

  // Never away, because a pool with no workers in it cannot run the work. On a host of fewer than a hundred workers
  // the arithmetic floors to zero and the clamp is the only reason this is runnable at all.
  assert_eq!(
    read("1%").resolve().get_workers().get(),
    (host().get() / 100).max(1),
    "one per cent of {} workers",
    host()
  );
}

#[test]
fn never_resolves_a_share_past_the_machine_it_is_a_share_of() {
  // Reading a command line rejects this with a message, but the variant is constructible directly, so the ceiling lives
  // where the number is turned into workers rather than only where text is turned into a request. A share that resolved
  // to several times the host would be oversubscription nobody asked for.
  let plan: ExecutionPlan = ExecutionRequest::Share(workers(500)).resolve();

  assert_eq!(plan.get_workers(), host());
}

#[test]
fn reports_a_plan_in_the_shape_a_report_carries() {
  let plan: ExecutionPlan = read("2").resolve();

  assert_eq!(
    serde_json::to_string(&plan).expect("a plan serializes"),
    r#"{"workers":2,"origin":"requested"}"#
  );

  assert_eq!(
    serde_json::to_string(&ExecutionRequest::Auto.resolve().get_origin()).expect("an origin serializes"),
    r#""auto""#
  );
}

#[cfg(feature = "rayon")]
mod bounds {
  use rayon::prelude::*;

  use super::{read, workers};

  /// The claim the whole policy rests on: one install bounds everything below it.
  ///
  /// Asserted through nested parallel iterators, because the bound is worth nothing if it holds only at the level that
  /// installed it. Work that fans out again inside a worker is where an unbounded pool would otherwise reappear.
  #[test]
  fn bounds_nested_parallel_work_by_the_plan() {
    let observed: usize = read("3")
      .resolve()
      .install(|| {
        (0..32)
          .into_par_iter()
          .map(|_| {
            (0..32)
              .into_par_iter()
              .map(|_| rayon::current_num_threads())
              .max()
              .expect("the inner range is not empty")
          })
          .max()
          .expect("the outer range is not empty")
      })
      .expect("three workers start");

    assert_eq!(observed, 3);
  }

  #[test]
  fn answers_with_what_the_operation_returned() {
    let plan = read("2").resolve();

    assert_eq!(plan.get_workers(), workers(2));
    assert_eq!(plan.install(|| "done").expect("two workers start"), "done");
  }
}
