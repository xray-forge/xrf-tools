use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::{
  TranslationLanguage, TranslationVerifier, TranslationVerifyLanguageSummary, TranslationVerifyOptions,
  TranslationVerifyResult,
};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::translations::lease::VERIFY_JOB_KIND;

/// What a completeness check reports back to the desktop surface.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationVerifySummary {
  /// Whether the run checked every source or was stopped between them.
  ///
  /// A stopped check reports the rows it reached; its silence about the rest is not a verdict.
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
///
/// Reads only. Nothing here writes, so an installation is a legitimate subject rather than a refusal.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_project"))]
#[tauri::command(rename = "verify_project")]
pub async fn translations_verify_project(
  registry: State<'_, Arc<JobRegistry>>,
  roots: XrayRoots,
  prefix: Option<String>,
  language: String,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationVerifySummary> {
  // `all` is accepted here, unlike the importer: a check reports every language at once, and that is
  // the run somebody opens this screen to make.
  let language: TranslationLanguage = TranslationLanguage::from_str(&language)?;

  log::info!("Verifying translations: {} root(s), '{language}'", roots.roots.len());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, VERIFY_JOB_KIND)
      .with_request(&json!({ "language": language.to_string() }))
      .with_progress(progress),
  )?;

  let options: TranslationVerifyOptions = TranslationVerifyOptions {
    job: job.clone(),
    is_strict: false,
    output: xrf_output::OutputOptions::default(),
    language,
    // The rows are the answer this surface shows. Building a finding per missing id would cost a
    // hundred thousand allocations to describe what the counts already say.
    is_detailed: false,
  };

  // Off the async worker: this parses every source in the project, which is 24,802 entries on an
  // Anomaly-sized import and not work an IPC executor should be holding.
  // Concluded with the summary rather than the crate's own result, because that is what this command answers: a window
  // that adopts this job after a reload reads the registry's copy and has to find the shape it would have been given.
  let outcome: TauriResult<TranslationVerifySummary> = tauri::async_runtime::spawn_blocking(move || {
    TranslationVerifier::verify_roots(&roots, prefix.as_deref(), &options)
  })
  .await
  .map_err(|error| format!("Translation check did not finish: {error}"))?
  .map_err(error_to_string)
  .map(|result: TranslationVerifyResult| TranslationVerifySummary {
    language: language.to_string(),
    outcome: result.outcome,
    checked: result.checked_translations_count,
    missing: result.missing_translations_count,
    languages: result.languages,
  });

  registration.conclude_with(&outcome, job.is_cancelled());

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
