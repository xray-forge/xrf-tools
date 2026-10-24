use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_job::JobProgress;
use xrf_pack::{ArchivePatchOptions, ArchivePatchResult, ArchivePatcher};
use xrf_utils::format_path;

use crate::ipc::archives::ARCHIVES_JOB_COMPARE;
use crate::patch::ArchivesPatchRequest;

/// Compares two roots without writing files.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "compare_archives"))]
#[tauri::command(rename = "compare_archives")]
pub async fn archives_compare_archives(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesPatchRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchivePatchResult> {
  let start: JobStart = JobStart::new(job_id, ARCHIVES_JOB_COMPARE)
    .with_request(&request)
    .with_kind_exclusion()
    .with_progress(progress);

  let ArchivesPatchRequest {
    config,
    is_verifying_payload,
    ..
  } = request;

  log::info!(
    "Comparing archives: {} -> {}",
    format_path(&config.input),
    config.target.as_deref().map_or_else(
      || String::from("its own loose gamedata"),
      |target| format_path(target).to_string()
    )
  );

  registry
    .run(
      &execution,
      start,
      move |job| {
        ArchivePatcher::compare_opt(
          &config,
          ArchivePatchOptions::default()
            .with_job(job)
            .with_verified_payloads(is_verifying_payload),
        )
      },
      |result| result.outcome,
    )
    .await
}
