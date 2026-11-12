use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_chunk::XRayByteOrder;
use xrf_job::{JobOutcome, JobProgress};
use xrf_rawdata::{LevelBuildExporter, LevelBuildRequest};
use xrf_utils::format_path;

use crate::ipc::levels::LEVELS_JOB_BUILD;
use crate::rawdata::level_build_result::LevelBuildResult;

/// Build a Level Editor scene into a level directory, as `level build` does, reporting each stage and stopping before
/// writing on request.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "build_level")]
pub(crate) async fn levels_build_level(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: LevelBuildRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LevelBuildResult> {
  let start: JobStart = JobStart::new(job_id, LEVELS_JOB_BUILD)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::tree(&request.destination)])
    .with_progress(progress);

  log::info!(
    "Building level: {} -> {}",
    format_path(&request.source),
    format_path(&request.destination)
  );

  registry
    .run(
      &execution,
      start,
      move |job| match LevelBuildExporter::build_to_directory::<XRayByteOrder>(&request, &job) {
        Ok(report) => Ok(LevelBuildResult {
          outcome: JobOutcome::Completed,
          report: Some(report),
        }),
        Err(_) if job.is_cancelled() => Ok(LevelBuildResult {
          outcome: JobOutcome::Cancelled,
          report: None,
        }),
        Err(error) => Err(error),
      },
      |result| result.outcome,
    )
    .await
}
