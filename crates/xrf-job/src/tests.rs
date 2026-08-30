use std::sync::Arc;
use std::time::Duration;

use crate::job_handle::JobHandle;
use crate::job_progress::JobProgress;
use crate::job_scope::JobScope;
use crate::progress_level::ProgressLevel;
use crate::progress_sink::RecordingSink;
use crate::progress_unit::ProgressUnit;

/// A handle reporting every unit, which is what an assertion over the sequence needs and what production never asks
/// for.
fn recording() -> (JobHandle, Arc<RecordingSink>) {
  let sink: Arc<RecordingSink> = Arc::new(RecordingSink::default());

  (JobHandle::with_interval(sink.clone(), Duration::ZERO), sink)
}

/// The level ids of one snapshot, which is what most of these assertions are actually about.
fn ids(progress: &JobProgress) -> Vec<&str> {
  progress.levels.iter().map(|level| level.id.as_str()).collect()
}

/// Every snapshot's stack, as ids.
fn reported_ids(sink: &RecordingSink) -> Vec<Vec<String>> {
  sink
    .list_reported()
    .iter()
    .map(|progress| progress.levels.iter().map(|level| level.id.clone()).collect())
    .collect()
}

#[test]
fn an_entered_level_is_reported_before_any_work_happens() {
  // A job that said nothing until its first unit would leave a reader on a blank spinner for as long as the first
  // entry takes, which for a large archive entry is exactly when it matters.
  let (job, sink) = recording();
  let _writing: JobScope = job.enter("write", Some(3));

  assert_eq!(reported_ids(&sink), vec![vec![String::from("write")]]);
}

#[test]
fn a_level_counts_its_own_units() {
  let (job, sink) = recording();
  let writing: JobScope = job.enter("write", Some(3));

  writing.advance();
  writing.advance();

  let reported: Vec<JobProgress> = sink.list_reported();
  let last: &JobProgress = reported.last().expect("a snapshot per advance");

  assert_eq!(last.levels[0].completed, 2);
  assert_eq!(last.levels[0].total, Some(3));
  assert_eq!(writing.completed(), 2);
}

#[test]
fn a_parent_counts_its_children_rather_than_their_units() {
  // The whole reason one mechanism serves "check 3 of 7" and "asset 400 of 40000": a parent's counts are its finished
  // children, not a sum of what they did.
  let (job, sink) = recording();
  let verifying: JobScope = job.enter("verify", Some(2));

  {
    let checking: JobScope = verifying.enter("textures", Some(40));

    checking.advance_by(40);
  }

  let reported: Vec<JobProgress> = sink.list_reported();
  let last: &JobProgress = reported.last().expect("a snapshot after the child left");

  assert_eq!(ids(last), vec!["verify"]);
  assert_eq!(last.levels[0].completed, 1);
  assert_eq!(last.levels[0].total, Some(2));
}

#[test]
fn a_snapshot_carries_the_whole_active_stack_at_one_instant() {
  // Three levels, which is the depth archives never reach and gamedata verification does: a run, a check, and what the
  // check is walking. Deep nesting shipping unexercised is how a generalization turns out to be wrong later.
  let (job, sink) = recording();
  let verifying: JobScope = job.enter("verify", Some(7));
  let checking: JobScope = verifying.enter("textures", Some(3));
  let walking: JobScope = checking.enter("assets", Some(40000));

  walking.advance();

  let reported: Vec<JobProgress> = sink.list_reported();
  let last: &JobProgress = reported.last().expect("a snapshot at depth");

  assert_eq!(ids(last), vec!["verify", "textures", "assets"]);
  assert_eq!(
    last
      .levels
      .iter()
      .map(|level| (level.completed, level.total))
      .collect::<Vec<(u64, Option<u64>)>>(),
    vec![(0, Some(7)), (0, Some(3)), (1, Some(40000))]
  );
}

#[test]
fn leaving_a_deep_level_unwinds_one_level_at_a_time() {
  let (job, sink) = recording();

  {
    let verifying: JobScope = job.enter("verify", Some(7));
    let checking: JobScope = verifying.enter("textures", Some(3));

    drop(checking);
    drop(verifying);
  }

  let stacks: Vec<Vec<String>> = reported_ids(&sink);

  // Entering reports twice, then the child's exit reports itself complete and its parent with it counted. The
  // outermost exit reports itself and then has no parent left to describe, which is a job between phases rather than a
  // job with none.
  assert_eq!(
    stacks,
    vec![
      vec![String::from("verify")],
      vec![String::from("verify"), String::from("textures")],
      vec![String::from("verify"), String::from("textures")],
      vec![String::from("verify")],
      vec![String::from("verify")],
    ]
  );
}

#[test]
fn an_uncountable_level_reports_no_total_rather_than_zero() {
  // Packing walks its source before it knows how much there is. A zero would render as a finished bar; absence renders
  // as the indeterminate state that is actually true.
  let (job, sink) = recording();
  let collecting: JobScope = job.enter("collect", None);

  collecting.advance();

  let reported: Vec<JobProgress> = sink.list_reported();
  let last: &JobProgress = reported.last().expect("a snapshot while collecting");

  assert_eq!(last.levels[0].total, None);
  assert_eq!(last.levels[0].completed, 1);
}

#[test]
fn a_level_can_count_bytes_instead_of_things() {
  let (job, _sink) = recording();
  let writing: JobScope = job.enter_bytes("write", Some(2048));

  writing.advance_by(1024);

  let progress: JobProgress = job.snapshot();

  assert_eq!(progress.levels[0].unit, ProgressUnit::Bytes);
  assert_eq!(progress.levels[0].completed, 1024);
}

#[test]
fn units_default_to_things() {
  let (job, _sink) = recording();
  let _writing: JobScope = job.enter("write", Some(1));

  assert_eq!(job.snapshot().levels[0].unit, ProgressUnit::Items);
}

#[test]
fn the_interval_bounds_how_often_units_are_reported() {
  // The rule that keeps a hundred thousand entries from flooding whatever is watching. Entering still reports, so the
  // count below is that one snapshot and no more.
  let sink: Arc<RecordingSink> = Arc::new(RecordingSink::default());
  let job: JobHandle = JobHandle::with_interval(sink.clone(), Duration::from_secs(3600));
  let writing: JobScope = job.enter("write", Some(10_000));

  for _ in 0..10_000 {
    writing.advance();
  }

  assert_eq!(sink.list_reported().len(), 1);
  assert_eq!(writing.completed(), 10_000);
}

#[test]
fn a_phase_change_is_never_swallowed_by_the_interval() {
  // A throttled unit is one a later snapshot supersedes; a throttled phase change leaves the wrong phase on screen for
  // as long as the next phase lasts.
  let sink: Arc<RecordingSink> = Arc::new(RecordingSink::default());
  let job: JobHandle = JobHandle::with_interval(sink.clone(), Duration::from_secs(3600));

  {
    let preparing: JobScope = job.enter("prepare", Some(1));

    preparing.advance();
  }

  let _writing: JobScope = job.enter("write", Some(1));

  let stacks: Vec<Vec<String>> = reported_ids(&sink);

  assert!(
    stacks.contains(&vec![String::from("prepare")]) && stacks.contains(&vec![String::from("write")]),
    "both phase transitions reported despite the interval, got {stacks:?}"
  );
}

#[test]
fn one_throttle_governs_the_whole_stack() {
  // A fast leaf must not multiply the rate for the levels above it: the throttle belongs to the run, not to a level.
  let sink: Arc<RecordingSink> = Arc::new(RecordingSink::default());
  let job: JobHandle = JobHandle::with_interval(sink.clone(), Duration::from_secs(3600));
  let outer: JobScope = job.enter("outer", Some(1));
  let inner: JobScope = outer.enter("inner", Some(10_000));

  for _ in 0..10_000 {
    inner.advance();
  }

  // Only the two entries reported; nothing the leaf did added a third.
  assert_eq!(sink.list_reported().len(), 2);
}

#[test]
fn detail_is_replaced_rather_than_accumulated() {
  let (job, _sink) = recording();
  let _writing: JobScope = job.enter("write", Some(2));

  job.set_detail(Some(String::from("configs\\system.ltx")));
  assert_eq!(job.snapshot().detail.as_deref(), Some("configs\\system.ltx"));

  job.set_detail(Some(String::from("meshes\\actor.ogf")));
  assert_eq!(job.snapshot().detail.as_deref(), Some("meshes\\actor.ogf"));

  job.set_detail(None);
  assert_eq!(job.snapshot().detail, None);
}

#[test]
fn a_job_between_levels_reports_nothing_rather_than_an_empty_stack() {
  let (job, sink) = recording();

  drop(job.enter("prepare", Some(1)));

  assert!(
    sink.list_reported().iter().all(|progress| !progress.levels.is_empty()),
    "a snapshot with no levels would describe a run with no phase as though it had one"
  );
}

#[test]
fn cancellation_is_visible_through_every_clone_of_the_handle() {
  // The app keeps one handle to cancel with and moves a clone onto a blocking thread. If the flag did not travel, the
  // control would do nothing at all.
  let (job, _sink) = recording();
  let working: JobHandle = job.clone();

  assert!(!working.is_cancelled());
  assert!(working.check_cancelled().is_ok());

  job.cancel();

  assert!(working.is_cancelled());
  assert!(working.check_cancelled().is_err());
}

#[test]
fn a_cancellation_check_yields_an_error_that_breaks_a_loop() {
  // The `?`-composing form exists because a parallel iterator stops on an error and on nothing else.
  let (job, _sink) = recording();
  let writing: JobScope = job.enter("write", Some(10));

  let outcome: Result<(), xrf_error::XrfError> = (0..10).try_for_each(|index| {
    job.check_cancelled()?;

    if index == 3 {
      job.cancel();
    }

    writing.advance();

    Ok(())
  });

  assert!(matches!(outcome, Err(xrf_error::XrfError::Cancelled { .. })));

  // Four entries were finished before the check refused the fifth, and none was left half done.
  assert_eq!(writing.completed(), 4);
}

#[test]
fn an_inert_job_counts_without_reporting() {
  // What the command line and every test that is not about progress hold. It still counts, so an operation reporting
  // its own totals needs no second path.
  let job: JobHandle = JobHandle::inert();
  let writing: JobScope = job.enter("write", Some(2));

  writing.advance();

  assert_eq!(writing.completed(), 1);
  assert!(!job.is_cancelled());
  assert_eq!(ids(&job.snapshot()), vec!["write"]);
}

#[test]
fn an_inert_job_can_still_be_cancelled_by_whoever_holds_it() {
  // Inert names where it reports, not whether it can be stopped. A lease-only run with no watcher is still a run
  // somebody may want to end.
  let job: JobHandle = JobHandle::inert();

  job.cancel();

  assert!(job.is_cancelled());
}

#[test]
fn a_reported_level_describes_the_counts_at_the_instant_it_was_taken() {
  // Snapshots are values, not views: a reader holding one must not watch it change underneath.
  let (job, sink) = recording();
  let writing: JobScope = job.enter("write", Some(3));

  writing.advance();

  let taken: Vec<ProgressLevel> = sink.list_reported().last().expect("a snapshot").levels.clone();

  writing.advance();

  assert_eq!(taken[0].completed, 1);
  assert_eq!(job.snapshot().levels[0].completed, 2);
}
