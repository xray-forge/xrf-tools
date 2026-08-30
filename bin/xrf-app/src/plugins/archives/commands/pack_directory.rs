use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchivePackConfig, ArchivePackOptions, ArchivePackResult, ArchivePacker};
use xrf_utils::format_path;

use crate::core::error::error_to_string;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::archives::lease::to_pack_lease_key;

/// Pack a directory into archive volumes from a configuration held by the caller.
///
/// Takes the whole configuration rather than a file path, so the editor packs exactly what is on screen
/// without having to save it first.
///
/// Holds its destination exclusively for the whole run, so a second request for the same output set is refused rather
/// than allowed to truncate the volumes this one is writing. A cancelled run answers with a result naming every volume
/// path it opened: those files exist and are incomplete, and nothing removes them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_directory"))]
#[tauri::command(rename = "pack_directory")]
pub async fn archives_pack_directory(
  registry: State<'_, Arc<JobRegistry>>,
  config: ArchivePackConfig,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchivePackResult> {
  log::info!(
    "Packing archive: {} -> {} as '{}'",
    format_path(&config.source),
    format_path(&config.destination),
    config.name
  );

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, "archives.pack")
      .with_lease_keys(vec![to_pack_lease_key(&config)])
      .with_request(&config)
      .with_progress(progress),
  )?;

  // Off the async worker: packing walks the whole source tree, compresses what the engine expects compressed, and
  // writes every volume. An `async fn` alone would leave all of that on an executor thread meant for short requests.
  let packing: JobHandle = job.clone();
  let outcome: TauriResult<ArchivePackResult> = tauri::async_runtime::spawn_blocking(move || {
    ArchivePacker::pack_opt(&config, ArchivePackOptions::default().with_job(packing))
  })
  .await
  .map_err(|error| format!("Archive pack did not finish: {error}"))?
  .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
