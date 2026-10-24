use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_job::JobProgress;
use xrf_ltx::{LtxFormatOptions, LtxProjectFormatResult};

use crate::ConfigsFormat;
use crate::ipc::configs::{CONFIGS_JOB_CHECK_FORMAT, CONFIGS_JOB_FORMAT};
use crate::request::ConfigsFormatRequest;

/// Report which LTX configs roots exposes are misformatted, in the format's own lane so a check never reads half a
/// rewrite.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "check_directory_format"))]
#[tauri::command(rename = "check_directory_format")]
pub async fn configs_check_directory_format(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ConfigsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  let start: JobStart = JobStart::new(job_id, CONFIGS_JOB_CHECK_FORMAT)
    .with_request(&request)
    .with_exclusion_group(CONFIGS_JOB_FORMAT.as_str())
    .with_progress(progress);

  let ConfigsFormatRequest { roots, prefix } = request;

  log::info!("Checking ltx format in {}", roots.describe());

  registry
    .run(
      &execution,
      start,
      move |job| ConfigsFormat::check(&roots, prefix.as_deref(), LtxFormatOptions::default().with_job(job)),
      |result| result.outcome,
    )
    .await
}
