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
use crate::plugins::archives::lease::COMPARE_JOB_KIND;
use crate::plugins::archives::request::ArchivesPatchRequest;

/// Compares two roots without writing files.
///
/// Ignores `is_forced` and takes no destination lease.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "compare_archives"))]
#[tauri::command(rename = "compare_archives")]
pub async fn archives_compare_archives(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesPatchRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchivePatchResult> {
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

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, COMPARE_JOB_KIND)
      .with_exclusion_group(COMPARE_JOB_KIND)
      .with_request(&config)
      .with_progress(progress),
  )?;

  // Off the async worker for the same reason a patch is: mounting two worlds and hashing what the sizes could not
  // settle is bounded only by how much the roots hold.
  run_job(
    &execution,
    "Archive comparison",
    registration,
    move || {
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
