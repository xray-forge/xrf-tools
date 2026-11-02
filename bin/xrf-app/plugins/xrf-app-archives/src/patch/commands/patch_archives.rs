use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_job::JobProgress;
use xrf_pack::{ArchivePatchOptions, ArchivePatchResult, ArchivePatcher};
use xrf_utils::format_path;

use crate::ipc::archives::ARCHIVES_JOB_PATCH;
use crate::lease::PUBLISH_ACTION_GROUP;
use crate::patch::ArchivesPatchRequest;

/// Publishes added and modified entries as patch volumes.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "patch_archives")]
pub async fn archives_patch_archives(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesPatchRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchivePatchResult> {
  let start: JobStart = JobStart::new(job_id, ARCHIVES_JOB_PATCH)
    .with_request(&request)
    .with_exclusion_group(PUBLISH_ACTION_GROUP)
    .with_resources(vec![JobResource::tree(&request.config.destination)])
    .with_progress(progress);

  let ArchivesPatchRequest {
    config,
    is_forced,
    is_verifying_payload,
  } = request;

  log::info!(
    "Patching archive: {} -> {} as '{}'",
    format_path(&config.input),
    config.target.as_deref().map_or_else(
      || String::from("its own loose gamedata"),
      |target| format_path(target).to_string()
    ),
    config.name
  );

  registry
    .run(
      &execution,
      start,
      move |job| {
        ArchivePatcher::patch_opt(
          &config,
          ArchivePatchOptions::default()
            .with_job(job)
            .with_force(is_forced)
            .with_verified_payloads(is_verifying_payload),
        )
      },
      |result| result.outcome,
    )
    .await
}
