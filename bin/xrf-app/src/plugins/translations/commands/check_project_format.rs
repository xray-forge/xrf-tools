use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::TranslationFormatResult;

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::translations::commands::format_project::run;
use crate::plugins::translations::lease::{CHECK_FORMAT_JOB_KIND, FORMAT_JOB_KIND};
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
  let TranslationsFormatRequest {
    directory,
    line_endings,
  } = request;

  log::info!("Checking translation source format in {}", directory.display());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, CHECK_FORMAT_JOB_KIND)
      .with_exclusion_group(FORMAT_JOB_KIND)
      .with_request(&json!({ "directory": directory, "lineEndings": line_endings }))
      .with_progress(progress),
  )?;

  let outcome: TauriResult<TranslationFormatResult> = run(&execution, job.clone(), directory, line_endings, true).await;

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
