use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_job::JobProgress;
use xrf_utils::format_path;

use crate::GamedataVerifySummary;
use crate::ipc::gamedata::GAMEDATA_JOB_VERIFY;
use crate::request::GamedataVerifyRequest;

/// Run the selected checks over a gamedata project, one verification at a time.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "verify_project")]
pub async fn gamedata_verify_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: GamedataVerifyRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<GamedataVerifySummary> {
  log::info!("Verifying gamedata project: {}", format_path(&request.root));

  let start: JobStart = JobStart::new(job_id, GAMEDATA_JOB_VERIFY)
    .with_request(&request)
    .with_kind_exclusion()
    .with_progress(progress);

  registry
    .run(
      &execution,
      start,
      move |job| GamedataVerifySummary::run(request, &job),
      |summary| summary.outcome,
    )
    .await
}
