use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_chunk::XRayByteOrder;
use xrf_job::{JobOutcome, JobProgress};
use xrf_rawdata::{LevelDecompileRequest, LevelSceneDecompiler};
use xrf_utils::format_path;

use crate::ipc::levels::LEVELS_JOB_DECOMPILE;
use crate::rawdata::level_decompile_result::LevelDecompileResult;

/// Decompile a compiled level's directory into a Level Editor scene on disk, as `level decompile` does, reporting each
/// step and stopping between steps on request.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "decompile_level")]
pub(crate) async fn levels_decompile_level(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: LevelDecompileRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LevelDecompileResult> {
  let start: JobStart = JobStart::new(job_id, LEVELS_JOB_DECOMPILE)
    .with_request(&request)
    .with_kind_exclusion()
    // Everything the decompile writes: the scene, its parts in the directory named after it, its objects and textures.
    .with_resources(vec![
      JobResource::file(&request.destination),
      JobResource::tree(request.destination.with_extension("")),
      JobResource::tree(request.get_objects_root()),
      JobResource::tree(request.get_textures_root()),
    ])
    .with_progress(progress);

  log::info!(
    "Decompiling level: {} -> {}",
    format_path(&request.source),
    format_path(&request.destination)
  );

  registry
    .run(
      &execution,
      start,
      move |job| match LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(
        &request,
        "xrf-app level decompiler",
        env!("CARGO_PKG_VERSION"),
        &job,
      ) {
        Ok(report) => Ok(LevelDecompileResult {
          outcome: JobOutcome::Completed,
          report: Some(report),
        }),
        Err(_) if job.is_cancelled() => Ok(LevelDecompileResult {
          outcome: JobOutcome::Cancelled,
          report: None,
        }),
        Err(error) => Err(error),
      },
      |result| result.outcome,
    )
    .await
}
