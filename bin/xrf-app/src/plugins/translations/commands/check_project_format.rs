use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::TranslationFormatResult;

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::translations::commands::format_project::run;
use crate::plugins::translations::request::TranslationsFormatRequest;

/// Report which JSON translation sources under a directory are not normalized.
///
/// Shares the formatter's exclusion group. Open editor sessions are allowed because checking does not rewrite files
/// or make their buffers stale; the separate job kind preserves that distinction in the reported outcome.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "check_project_format"))]
#[tauri::command(rename = "check_project_format")]
pub async fn translations_check_project_format(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationFormatResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::TranslationsCheckFormat).with_request(&request);

  let TranslationsFormatRequest {
    directory,
    line_endings,
  } = request;

  log::info!("Checking translation source format in {}", directory.display());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::TranslationsFormat.as_str())
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Translations formatting",
    registration,
    move || run(job, directory, line_endings, true),
    |result| result.outcome,
  )
  .await
}
