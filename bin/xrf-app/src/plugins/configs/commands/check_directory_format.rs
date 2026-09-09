use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::configs::ltx_roots::open_ltx_project;
use crate::plugins::configs::request::ConfigsFormatRequest;

/// Report which LTX configs roots exposes are misformatted.
///
/// Reads archived configs too. Shares the formatter's exclusion group while retaining a separate job kind because
/// checking reports findings without rewriting files.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "check_directory_format"))]
#[tauri::command(rename = "check_directory_format")]
pub async fn configs_check_directory_format(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ConfigsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ConfigsCheckFormat).with_request(&request);

  let ConfigsFormatRequest { roots, prefix } = request;

  log::info!("Checking ltx format in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::ConfigsFormat.as_str())
      .with_progress(progress),
  )?;

  // Off the async worker: this mounts every root and reads every config it holds.
  run_job(
    &execution,
    "Configs format check",
    registration,
    move || {
      let project: LtxProject = open_ltx_project(&roots, prefix.as_deref(), Default::default())?;

      project.check_format_all_files_opt(LtxFormatOptions::default().with_job(job))
    },
    |result| result.outcome,
  )
  .await
}
