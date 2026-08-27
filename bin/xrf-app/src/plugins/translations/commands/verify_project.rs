use std::str::FromStr;

use serde::{Deserialize, Serialize};
use xrf_translation::{
  ProjectVerifyLanguageSummary, ProjectVerifyOptions, ProjectVerifyResult, TranslationLanguage, verify_roots,
};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// What a completeness check reports back to the desktop surface.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationVerifySummary {
  /// The language the check was narrowed to, or `all`.
  pub language: String,
  /// Ids checked across every source.
  pub checked: u32,
  /// Ids with no text, counted once per language that lacks them.
  pub missing: u32,
  pub languages: Vec<ProjectVerifyLanguageSummary>,
}

/// Report which translations are missing from which languages.
///
/// Reads only. Nothing here writes, so an installation is a legitimate subject rather than a refusal.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_project"))]
#[tauri::command(rename = "verify_project")]
pub async fn translations_verify_project(
  roots: XrayRoots,
  prefix: Option<String>,
  language: String,
) -> TauriResult<TranslationVerifySummary> {
  // `all` is accepted here, unlike the importer: a check reports every language at once, and that is
  // the run somebody opens this screen to make.
  let language: TranslationLanguage = TranslationLanguage::from_str(&language)?;

  log::info!("Verifying translations: {} root(s), '{language}'", roots.roots.len());

  let options: ProjectVerifyOptions = ProjectVerifyOptions {
    is_strict: false,
    output: xrf_output::OutputOptions::default(),
    language,
    // The rows are the answer this surface shows. Building a finding per missing id would cost a
    // hundred thousand allocations to describe what the counts already say.
    is_detailed: false,
  };

  // Off the async worker: this parses every source in the project, which is 24,802 entries on an
  // Anomaly-sized import and not work an IPC executor should be holding.
  let result: ProjectVerifyResult =
    tauri::async_runtime::spawn_blocking(move || verify_roots(&roots, prefix.as_deref(), &options))
      .await
      .map_err(|error| format!("Translation check did not finish: {error}"))?
      .map_err(error_to_string)?;

  log::info!(
    "Verified {} translation(s), {} missing across {} language row(s)",
    result.checked_translations_count,
    result.missing_translations_count,
    result.languages.len()
  );

  Ok(TranslationVerifySummary {
    language: language.to_string(),
    checked: result.checked_translations_count,
    missing: result.missing_translations_count,
    languages: result.languages,
  })
}
