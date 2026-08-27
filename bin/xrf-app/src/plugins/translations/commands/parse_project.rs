use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_translation::{ProjectParseCensus, ProjectParseOptions, ProjectParseResult, TranslationLanguage};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// What an import run reports back to the desktop surface.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationParseSummary {
  /// The language every entry this run read was filed under.
  pub language: String,
  /// Whether the run computed its answer without writing it.
  pub is_dry_run: bool,
  pub census: ProjectParseCensus,
  pub findings: Vec<TranslationParseFinding>,
}

/// One thing worth reporting about a file the run met.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationParseFinding {
  pub rule: String,
  pub subject: Option<String>,
  pub message: String,
}

/// Import one language's raw XML string tables into JSON sources.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "parse_project"))]
#[tauri::command(rename = "parse_project")]
pub async fn translations_parse_project(
  roots: XrayRoots,
  language: String,
  prefix: Option<String>,
  output_dir: PathBuf,
  file: Option<String>,
  is_overwrite: bool,
  is_dry_run: bool,
) -> TauriResult<TranslationParseSummary> {
  let language: TranslationLanguage = TranslationLanguage::from_str_single(&language).map_err(error_to_string)?;

  log::info!(
    "Parsing translations: {} root(s), '{language}', into {}{}",
    roots.roots.len(),
    output_dir.display(),
    if is_dry_run { " (dry run)" } else { "" }
  );

  let options: ProjectParseOptions = ProjectParseOptions {
    output: xrf_output::OutputOptions::default(),
    roots,
    prefix,
    language,
    output_dir,
    file,
    is_overwrite,
    is_dry_run,
  };

  // Off the async worker, because this reads and writes a whole tree: 134 files and 24,000 entries on
  // an Anomaly-sized import, which is not work an IPC executor should be holding.
  let result: ProjectParseResult =
    tauri::async_runtime::spawn_blocking(move || xrf_translation::parse_translations(&options))
      .await
      .map_err(|error| format!("Translation import did not finish: {error}"))?
      .map_err(error_to_string)?;

  log::info!(
    "Parsed {} translation file(s), {} created, {} updated, {} finding(s)",
    result.census.files_read,
    result.census.files_created,
    result.census.files_updated,
    result.get_findings().len()
  );

  Ok(TranslationParseSummary {
    language: result.language.clone(),
    is_dry_run: result.is_dry_run,
    census: result.census.clone(),
    findings: result
      .get_findings()
      .iter()
      .map(|finding| TranslationParseFinding {
        rule: finding.rule_id().to_string(),
        subject: finding.subject().map(String::from),
        message: finding.message().to_string(),
      })
      .collect(),
  })
}
