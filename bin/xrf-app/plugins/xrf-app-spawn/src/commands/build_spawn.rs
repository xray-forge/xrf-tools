use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_job::JobProgress;
use xrf_spawn_builder::SpawnRootRequest;
use xrf_utils::format_path;

use crate::build::{SpawnBuild, SpawnBuildResult};

/// Build a root spawn from a game's levels and write it, as `spawn build` does, as a job a cancel stops before it
/// writes.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "build_spawn")]
pub(crate) async fn spawn_build_spawn(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: SpawnRootRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<SpawnBuildResult> {
  log::info!(
    "Building spawn: {} from {}",
    format_path(&request.get_spawn_path()),
    format_path(&request.get_levels_root())
  );

  registry
    .run(
      &execution,
      SpawnBuild::start(&request, job_id, progress),
      move |job| SpawnBuild::build(&request, &job),
      |result| result.outcome,
    )
    .await
}
