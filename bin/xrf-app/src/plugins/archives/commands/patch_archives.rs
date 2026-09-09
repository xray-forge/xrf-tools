use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchivePatchOptions, ArchivePatchResult, ArchivePatcher};
use xrf_utils::format_path;

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::archives::lease::{PATCH_JOB_KIND, PUBLISH_ACTION_GROUP, to_published_set_lease_key};
use crate::plugins::archives::request::ArchivesPatchRequest;

/// Publishes added and modified entries as patch volumes.
///
/// Holds an exclusive destination lease and shares the publishing group with archive packing. Replacing an
/// existing set requires `is_forced`.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "patch_archives"))]
#[tauri::command(rename = "patch_archives")]
pub async fn archives_patch_archives(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesPatchRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchivePatchResult> {
  let ArchivesPatchRequest {
    config,
    is_forced,
    is_strict,
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

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, PATCH_JOB_KIND)
      .with_exclusion_group(PUBLISH_ACTION_GROUP)
      .with_lease_keys(vec![to_published_set_lease_key(&config.destination, &config.name)])
      .with_request(&config)
      .with_progress(progress),
  )?;

  // Off the async worker: a patch mounts two worlds, reads every payload a decision needs, and then writes volumes.
  // An `async fn` alone would leave all of that on an executor thread meant for short requests.
  run_job(
    &execution,
    "Archive patch",
    registration,
    move || {
      ArchivePatcher::patch_opt(
        &config,
        ArchivePatchOptions::default()
          .with_job(job)
          .with_force(is_forced)
          .with_strict(is_strict)
          .with_verified_payloads(is_verifying_payload),
      )
    },
    |result| result.outcome,
  )
  .await
}
