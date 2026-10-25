use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_job::JobProgress;
use xrf_ltx::{LtxFormatOptions, LtxProjectFormatResult};

use crate::ConfigsFormat;
use crate::ipc::configs::CONFIGS_JOB_FORMAT;
use crate::request::ConfigsFormatRequest;

/// Rewrite the LTX configs roots exposes, as one commit, one format or format check at a time.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "format_directory")]
pub async fn configs_format_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  assets: State<'_, AssetMountState>,
  request: ConfigsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  let start: JobStart = JobStart::new(job_id, CONFIGS_JOB_FORMAT)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(
      request
        .roots
        .roots
        .iter()
        .map(|root| JobResource::tree(&root.path))
        .collect(),
    )
    .with_progress(progress);

  let ConfigsFormatRequest { roots, prefix } = request;

  log::info!("Formatting ltx configs in {}", roots.describe());

  let assets: AssetMountState = AssetMountState::clone(&assets);

  registry
    .run(
      &execution,
      start,
      move |job| {
        ConfigsFormat::format(
          &assets,
          &roots,
          prefix.as_deref(),
          LtxFormatOptions::default().with_job(job),
        )
      },
      |result| result.outcome,
    )
    .await
}
