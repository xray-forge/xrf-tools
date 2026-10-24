use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_job::JobProgress;
use xrf_translation::TranslationFormatResult;

use crate::ipc::translations::TRANSLATIONS_JOB_FORMAT;
use crate::request::TranslationsFormatRequest;
use crate::{TranslationFormatGuard, TranslationProjects, TranslationsFormatRun};

/// Normalize the JSON translation sources under a directory, refused while an open project holds any of them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_project"))]
#[tauri::command(rename = "format_project")]
pub async fn translations_format_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  projects: State<'_, TranslationProjects>,
  request: TranslationsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationFormatResult> {
  let start: JobStart = JobStart::new(job_id, TRANSLATIONS_JOB_FORMAT)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::tree(&request.directory)])
    .with_progress(progress);

  let TranslationsFormatRequest {
    directory,
    line_endings,
  } = request;

  log::info!("Formatting translation sources in {}", directory.display());

  // Before the job exists, because this is a refusal rather than a failed run.
  TranslationFormatGuard::require_none_over(&projects, &directory)?;

  registry
    .run(
      &execution,
      start,
      move |job| TranslationsFormatRun::format(job, directory, line_endings),
      |result| result.outcome,
    )
    .await
}
