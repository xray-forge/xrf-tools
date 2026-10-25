use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_job::JobProgress;
use xrf_translation::TranslationFormatResult;

use crate::TranslationsFormatRun;
use crate::ipc::translations::{TRANSLATIONS_JOB_CHECK_FORMAT, TRANSLATIONS_JOB_FORMAT};
use crate::request::TranslationsFormatRequest;

/// Report which JSON translation sources under a directory are not normalized, in the format's own lane so a check
/// never reads half a rewrite.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "check_project_format")]
pub async fn translations_check_project_format(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationFormatResult> {
  let start: JobStart = JobStart::new(job_id, TRANSLATIONS_JOB_CHECK_FORMAT)
    .with_request(&request)
    .with_exclusion_group(TRANSLATIONS_JOB_FORMAT.as_str())
    .with_progress(progress);

  let TranslationsFormatRequest {
    directory,
    line_endings,
  } = request;

  log::info!("Checking translation source format in {}", directory.display());

  registry
    .run(
      &execution,
      start,
      move |job| TranslationsFormatRun::check(job, directory, line_endings),
      |result| result.outcome,
    )
    .await
}
