use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::jobs::{JobResource, JobStart};
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobOutcome, JobProgress};
use xrf_ltx::LtxDialect;
use xrf_spawn_builder::{SpawnRootGamedata, SpawnRootRequest};

use crate::build::SpawnBuildResult;
use crate::ipc::spawn::SPAWN_JOB_BUILD;

/// A root spawn build as a job, as `spawn build` runs it: one at a time, holding the spawn file it writes, cancellable
/// between stages but not once it writes.
pub(crate) struct SpawnBuild;

impl SpawnBuild {
  /// The job's registry entry: its kind, run one at a time, and the spawn file it holds.
  pub(crate) fn start(request: &SpawnRootRequest, job_id: Uuid, progress: Channel<JobProgress>) -> JobStart {
    JobStart::new(job_id, SPAWN_JOB_BUILD)
      .with_request(request)
      .with_kind_exclusion()
      .with_resources(vec![JobResource::file(request.get_spawn_path())])
      .with_progress(progress)
  }

  /// Builds and writes the spawn on the job's thread, reading the game's configs under `dialect`; a build that fails
  /// after a cancel ends as cancelled, nothing written.
  ///
  /// # Errors
  ///
  /// Returns the [`SpawnRootGamedata::build`] error where the job was not cancelled.
  pub(crate) fn build(
    request: &SpawnRootRequest,
    dialect: &dyn LtxDialect,
    job: &JobHandle,
  ) -> XrfResult<SpawnBuildResult> {
    match SpawnRootGamedata::build(request, dialect, job) {
      Ok(spawn) => Ok(SpawnBuildResult {
        outcome: JobOutcome::Completed,
        spawn: Some(spawn),
      }),
      Err(_) if job.is_cancelled() => Ok(SpawnBuildResult {
        outcome: JobOutcome::Cancelled,
        spawn: None,
      }),
      Err(error) => Err(error),
    }
  }
}

#[cfg(test)]
mod tests;
