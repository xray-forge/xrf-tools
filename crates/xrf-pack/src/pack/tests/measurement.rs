//! What a run reports about its own clock.

use std::time::Duration;

use xrf_job::{JobHandle, JobOutcome};

use crate::pack::tests::fixtures::{BINARY, CONFIG, create_config, pack};
use crate::pack::{ArchivePackOptions, ArchivePackResult, ArchivePacker};

/// The three phases as one total, which is what a reader adding them up gets.
fn sum_phases(result: &ArchivePackResult) -> Duration {
  result.collect_duration + result.write_duration + result.finalize_duration
}

#[test]
fn a_finished_run_divides_its_duration_between_its_phases() {
  let (result, _) = pack(
    "a_finished_run_divides_its_duration_between_its_phases",
    &[("configs\\system.ltx", CONFIG), ("textures\\wall.dds", BINARY)],
    |_| {},
  );

  assert_eq!(result.outcome, JobOutcome::Completed);
  assert_eq!(
    sum_phases(&result),
    result.duration,
    "the phases tile the run rather than sampling it"
  );
}

#[test]
fn a_run_stopped_before_writing_reports_only_the_walk() {
  let (config, _) = create_config(
    "a_run_stopped_before_writing_reports_only_the_walk",
    &[("configs\\system.ltx", CONFIG)],
  );
  let job: JobHandle = JobHandle::default();

  // Cancelled before the run starts, so it reaches the boundary after the walk and stops there.
  job.cancel();

  let result: ArchivePackResult =
    ArchivePacker::pack_opt(&config, ArchivePackOptions::default().with_job(job)).expect("a cancelled run reports");

  assert_eq!(result.outcome, JobOutcome::Cancelled);
  assert_eq!(sum_phases(&result), result.duration);
  assert_eq!(
    result.finalize_duration,
    Duration::ZERO,
    "a run that published nothing spent no time naming a set"
  );
}
