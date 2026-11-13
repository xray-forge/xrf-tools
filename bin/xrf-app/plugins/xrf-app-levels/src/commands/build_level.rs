use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome, JobProgress};
use xrf_level_compiler::{LevelCompileRequest, LevelCompiler};
use xrf_rawdata::{LevelBuildExporter, LevelBuildRequest};
use xrf_utils::format_path;

use crate::ipc::levels::LEVELS_JOB_BUILD;
use crate::rawdata::level_build_result::LevelBuildResult;
use crate::rawdata::level_compiler_libraries::LevelCompilerLibraries;

/// Build a Level Editor scene into a level directory, as `level build` does, then compile it where `compile` names
/// the steps, as `level compile` does, one job reporting both. A cancel stops the build between its steps and kills a
/// running compiler. The compile works on the directory the build wrote, whatever level `compile` names, overwrites
/// what it writes as the build does where the build is forced, and lights the details on D3D12 with the DXC the app
/// bundles where it traces on the GPU.
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
  let compile: Option<LevelCompileRequest> = compile.map(|compile| LevelCompileRequest {
    level: request.destination.clone(),
    is_forced: request.is_forced,
    dxc: libraries.dxc.clone(),
    ..compile
  });
  let mut resources: Vec<JobResource> = vec![JobResource::tree(&request.destination)];

  if let Some(work) = compile.as_ref().and_then(|compile| compile.work.as_ref()) {
    resources.push(JobResource::tree(work));
  }

  let start: JobStart = JobStart::new(job_id, LEVELS_JOB_BUILD)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(resources)
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
      move |job| build_and_compile(&request, compile.as_ref(), &job),
      |result| result.outcome,
    )
    .await
}

/// What the build wrote and the compile after it ran, on the job's thread; cancelled where the job was cancelled before
/// either finished, the build's report kept where it was the compile that stopped.
fn build_and_compile(
  request: &LevelBuildRequest,
  compile: Option<&LevelCompileRequest>,
  job: &JobHandle,
) -> XrfResult<LevelBuildResult> {
  let cancelled = |report| LevelBuildResult {
    outcome: JobOutcome::Cancelled,
    report,
    compile: None,
  };
  let report = match LevelBuildExporter::build_to_directory::<XRayByteOrder>(request, job) {
    Ok(report) => report,
    Err(_) if job.is_cancelled() => return Ok(cancelled(None)),
    Err(error) => return Err(error),
  };
  let compiled = match compile.map(|compile| LevelCompiler::compile(compile, job)).transpose() {
    Ok(compiled) => compiled,
    Err(_) if job.is_cancelled() => return Ok(cancelled(Some(report))),
    Err(error) => {
      return Err(XrfError::new_invalid_error(format!(
        "The level was built into {}, then its compile stopped: {error}",
        report.destination
      )));
    }
  };

  Ok(LevelBuildResult {
    outcome: JobOutcome::Completed,
    report: Some(report),
    compile: compiled,
  })
}
