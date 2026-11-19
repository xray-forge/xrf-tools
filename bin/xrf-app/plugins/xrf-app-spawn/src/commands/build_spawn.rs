use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_dltx::DltxDialect;
use xrf_job::JobProgress;
use xrf_ltx::LtxDialect;
use xrf_spawn_builder::SpawnRootRequest;
use xrf_utils::format_path;

use crate::build::{SpawnBuild, SpawnBuildResult};

/// Build and write a root spawn from a game's levels, as `spawn build` does, as a job cancellable until it writes. The
/// game's configs are read with the DLTX patch dialect where `is_dltx` asks, as `spawn build --dltx` does.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "build_spawn")]
pub(crate) async fn spawn_build_spawn(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: SpawnRootRequest,
  is_dltx: bool,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<SpawnBuildResult> {
  log::info!(
    "Building spawn: {} from {}",
    format_path(&request.get_spawn_path()),
    format_path(&request.get_levels_root())
  );

  let dialect: Arc<dyn LtxDialect> = DltxDialect::select(is_dltx);

  registry
    .run(
      &execution,
      SpawnBuild::start(&request, job_id, progress),
      move |job| SpawnBuild::build(&request, dialect.as_ref(), &job),
      |result| result.outcome,
    )
    .await
}
