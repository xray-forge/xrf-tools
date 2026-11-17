use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::jobs::{JobResource, JobStart};
use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobOutcome, JobProgress};
use xrf_spawn_builder::{SpawnRootGamedata, SpawnRootRequest};

use crate::build::SpawnBuildResult;
use crate::ipc::spawn::SPAWN_JOB_BUILD;

/// A root spawn built from a game's levels as a job, as `spawn build` builds it: one build at a time, holding the spawn
/// file it writes, stopped between its stages and never once it writes.
pub(crate) struct SpawnBuild;

impl SpawnBuild {
  /// The job's place in the registry: its kind, run one at a time, and the spawn file it writes.
  pub(crate) fn start(request: &SpawnRootRequest, job_id: Uuid, progress: Channel<JobProgress>) -> JobStart {
    JobStart::new(job_id, SPAWN_JOB_BUILD)
      .with_request(request)
      .with_kind_exclusion()
      .with_resources(vec![JobResource::file(request.get_spawn_path())])
      .with_progress(progress)
  }

  /// Builds and writes the spawn on the job's thread; cancelled, with nothing written, where the job was cancelled
  /// before the build wrote, whatever stopped it.
  ///
  /// # Errors
  ///
  /// Returns why the build refused ([`SpawnRootGamedata::build`]) where the job was not cancelled.
  pub(crate) fn build(request: &SpawnRootRequest, job: &JobHandle) -> XrfResult<SpawnBuildResult> {
    match SpawnRootGamedata::build(request, job) {
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
