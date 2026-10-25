use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_job::JobProgress;
use xrf_pack::{ArchivePackOptions, ArchivePackResult, ArchivePacker};
use xrf_utils::format_path;

use crate::ipc::archives::ARCHIVES_JOB_PACK;
use crate::lease::PUBLISH_ACTION_GROUP;
use crate::pack::ArchivesPackRequest;

/// Packs a directory using the supplied configuration.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "pack_directory")]
pub async fn archives_pack_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesPackRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchivePackResult> {
  let start: JobStart = JobStart::new(job_id, ARCHIVES_JOB_PACK)
    .with_request(&request)
    .with_exclusion_group(PUBLISH_ACTION_GROUP)
    .with_resources(vec![JobResource::tree(&request.config.destination)])
    .with_progress(progress);

  let ArchivesPackRequest { config, is_forced } = request;

  log::info!(
    "Packing archive: {} -> {} as '{}'",
    format_path(&config.source),
    format_path(&config.destination),
    config.name
  );

  registry
    .run(
      &execution,
      start,
      move |job| {
        ArchivePacker::pack_opt(
          &config,
          ArchivePackOptions::default().with_job(job).with_force(is_forced),
        )
      },
      |result| result.outcome,
    )
    .await
}
