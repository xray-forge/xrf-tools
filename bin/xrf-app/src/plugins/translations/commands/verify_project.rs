use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::{
  TranslationLanguage, TranslationVerifier, TranslationVerifyLanguageSummary, TranslationVerifyOptions,
  TranslationVerifyResult,
};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::translations::request::TranslationsVerifyRequest;

/// What a completeness check reports back to the desktop surface.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationVerifySummary {
  /// Whether the run checked every source or was stopped between them.
  pub outcome: xrf_job::JobOutcome,
  /// The language the check was narrowed to, or `all`.
  pub language: String,
  /// Ids checked across every source.
  pub checked: u32,
  /// Ids with no text, counted once per language that lacks them.
  pub missing: u32,
  pub languages: Vec<TranslationVerifyLanguageSummary>,
}

/// Report which translations are missing from which languages.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_project"))]
#[tauri::command(rename = "verify_project")]
pub async fn translations_verify_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationsVerifyRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationVerifySummary> {
  let start: JobStart = JobStart::new(job_id, JobKind::TranslationsVerify).with_request(&request);

  let TranslationsVerifyRequest {
    roots,
    prefix,
    language,
  } = request;

  let language: TranslationLanguage = TranslationLanguage::from_str(&language)?;

  log::info!("Verifying translations: {} root(s), '{language}'", roots.roots.len());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::TranslationsVerify.as_str())
      .with_progress(progress),
  )?;

  let options: TranslationVerifyOptions = TranslationVerifyOptions {
    job,
    is_strict: false,
    output: xrf_output::OutputOptions::default(),
    language,
    is_detailed: false,
  };

  let outcome: TauriResult<TranslationVerifySummary> = run_job(
    &execution,
    "Translation check",
    registration,
    move || {
      TranslationVerifier::verify_roots(&roots, prefix.as_deref(), &options).map(|result: TranslationVerifyResult| {
        TranslationVerifySummary {
          language: language.to_string(),
          outcome: result.outcome,
          checked: result.checked_translations_count,
          missing: result.missing_translations_count,
          languages: result.languages,
        }
      })
    },
    |summary| summary.outcome,
  )
  .await;

  if let Ok(summary) = &outcome {
    log::info!(
      "Verified {} translation(s), {} missing across {} language row(s)",
      summary.checked,
      summary.missing,
      summary.languages.len()
    );
  }

  outcome
}
