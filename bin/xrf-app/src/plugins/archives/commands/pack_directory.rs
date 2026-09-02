use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchivePackConfig, ArchivePackOptions, ArchivePackResult, ArchivePacker};
use xrf_utils::format_path;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::archives::lease::{PACK_JOB_KIND, to_pack_lease_key};

/// Pack a directory into archive volumes from a configuration held by the caller.
///
/// Takes the whole configuration rather than a file path, so the editor packs exactly what is on screen
/// without having to save it first.
///
/// Holds its destination exclusively for the whole run, so a second request for the same output set is refused rather
/// than allowed to truncate the volumes this one is writing.
///
/// `is_forced` is the user answering for a destination that already holds this set; without it such a run is refused
/// before anything is written. It also decides what a stopped run leaves: an unforced one takes back the volumes it
/// made and the destination is untouched, while a forced one cannot tell its own output from what it replaced and
/// answers with a result naming every volume path it opened.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_directory"))]
#[tauri::command(rename = "pack_directory")]
pub async fn archives_pack_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  config: ArchivePackConfig,
  is_forced: bool,
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
    JobStart::new(job_id, PACK_JOB_KIND)
      .with_lease_keys(vec![to_pack_lease_key(&config)])
      .with_request(&config)
      .with_progress(progress),
  )?;

  // Off the async worker: packing walks the whole source tree, compresses what the engine expects compressed, and
  // writes every volume. An `async fn` alone would leave all of that on an executor thread meant for short requests.
  let packing: JobHandle = job.clone();
  let outcome: TauriResult<ArchivePackResult> = execution
    .run_blocking("Archive pack", move || {
      ArchivePacker::pack_opt(
        &config,
        ArchivePackOptions::default().with_job(packing).with_force(is_forced),
      )
    })
    .await?
    .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
