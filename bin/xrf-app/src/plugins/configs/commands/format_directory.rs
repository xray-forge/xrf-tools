use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsFormatRequest;

/// Rewrite the LTX configs roots exposes.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_directory"))]
#[tauri::command(rename = "format_directory")]
pub async fn configs_format_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ConfigsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ConfigsFormat).with_request(&request);

  let ConfigsFormatRequest { roots, prefix } = request;

  log::info!("Formatting ltx configs in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::ConfigsFormat.as_str())
      .with_resources(roots.roots.iter().map(|root| JobResource::tree(&root.path)).collect())
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Configs formatting",
    registration,
    move || {
      let project: LtxProject = LtxProject::open_at_roots_opt(&roots, prefix.as_deref(), Default::default())?;

      project.format_all_files_opt(LtxFormatOptions::default().with_job(job))
    },
    |result| result.outcome,
  )
  .await
}
