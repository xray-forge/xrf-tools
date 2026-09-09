use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchivePackOptions, ArchivePackResult, ArchivePacker};
use xrf_utils::format_path;

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::archives::lease::PUBLISH_ACTION_GROUP;
use crate::plugins::archives::request::ArchivesPackRequest;

/// Packs a directory using the supplied configuration.
///
/// Holds an exclusive destination lease. Replacing an existing set requires `is_forced`. Unforced runs roll
/// back on failure or cancellation; forced runs cannot restore overwritten volumes.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_directory"))]
#[tauri::command(rename = "pack_directory")]
pub async fn archives_pack_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesPackRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchivePackResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ArchivesPack).with_request(&request);

  let ArchivesPackRequest { config, is_forced } = request;

  log::info!(
    "Packing archive: {} -> {} as '{}'",
    format_path(&config.source),
    format_path(&config.destination),
    config.name
  );

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(PUBLISH_ACTION_GROUP)
      .with_resources(vec![JobResource::tree(&config.destination)])
      .with_progress(progress),
  )?;

  // Off the async worker: packing walks the whole source tree, compresses what the engine expects compressed, and
  // writes every volume. An `async fn` alone would leave all of that on an executor thread meant for short requests.
  run_job(
    &execution,
    "Archive pack",
    registration,
    move || {
      ArchivePacker::pack_opt(
        &config,
        ArchivePackOptions::default().with_job(job).with_force(is_forced),
      )
    },
    |result| result.outcome,
  )
  .await
}
