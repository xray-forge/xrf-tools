use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_job::JobProgress;
use xrf_level_compiler::LevelCompileRequest;
use xrf_rawdata::LevelBuildRequest;
use xrf_utils::format_path;

use crate::ipc::levels::LEVELS_JOB_BUILD;
use crate::rawdata::level_build_result::LevelBuildResult;
use crate::rawdata::level_build_run::LevelBuildRun;
use crate::rawdata::level_compiler_libraries::LevelCompilerLibraries;

/// Build a Level Editor scene into a level directory, as `level build` does, then compile it when `compile` is given,
/// as `level compile` does, in one job.
///
/// A cancel stops the build between steps and the compile within its running step. The compile always works on the
/// directory the build wrote and is forced when the build is ([`LevelBuildRun::new`]).
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "build_level")]
pub(crate) async fn levels_build_level(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  libraries: State<'_, LevelCompilerLibraries>,
  request: LevelBuildRequest,
  compile: Option<LevelCompileRequest>,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LevelBuildResult> {
  let run: LevelBuildRun = LevelBuildRun::new(request, compile, libraries.dxc.clone());
  let start: JobStart = JobStart::new(job_id, LEVELS_JOB_BUILD)
    .with_request(&run.build)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::tree(&run.build.destination)])
    .with_progress(progress);

  log::info!(
    "Building level: {} -> {}",
    format_path(&run.build.source),
    format_path(&run.build.destination)
  );

  registry
    .run(&execution, start, move |job| run.run(&job), |result| result.outcome)
    .await
}
