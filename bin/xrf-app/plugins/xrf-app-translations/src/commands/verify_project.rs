use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_job::JobProgress;
use xrf_translation::{TranslationVerifier, TranslationVerifyOptions, TranslationVerifyResult};

use crate::ipc::translations::TRANSLATIONS_JOB_VERIFY;
use crate::request::TranslationsVerifyRequest;

/// Report which translations are missing from which languages, one check at a time.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_project"))]
#[tauri::command(rename = "verify_project")]
pub async fn translations_verify_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationsVerifyRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationVerifyResult> {
  let start: JobStart = JobStart::new(job_id, TRANSLATIONS_JOB_VERIFY)
    .with_request(&request)
    .with_kind_exclusion()
    .with_progress(progress);

  let TranslationsVerifyRequest {
    roots,
    prefix,
    language,
  } = request;

  log::info!("Verifying translations: {} root(s), '{language}'", roots.roots.len());

  let result: TranslationVerifyResult = registry
    .run(
      &execution,
      start,
      move |job| {
        TranslationVerifier::verify_roots(
          &roots,
          prefix.as_deref(),
          &TranslationVerifyOptions {
            job,
            is_strict: false,
            output: xrf_output::OutputOptions::default(),
            language,
            is_detailed: false,
          },
        )
      },
      |result| result.outcome,
    )
    .await?;

  log::info!(
    "Verified {} translation(s), {} missing across {} language row(s)",
    result.checked_translations_count,
    result.missing_translations_count,
    result.languages.len()
  );

  Ok(result)
}
