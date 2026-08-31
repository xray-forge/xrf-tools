use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::{ProjectParseCensus, ProjectParseOptions, ProjectParseResult, TranslationLanguage};
use xrf_utils::format_path;
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::translations::lease::{PARSE_JOB_KIND, to_output_lease_key};

/// What an import was asked to do.
///
/// One argument rather than seven, because a Tauri command's parameters are its wire signature and seven of them plus
/// a job's own two is more than a reader can hold. It is also exactly what the registry retains, so a window adopting
/// this run after a reload sees the request rather than a summary of it.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationParseRequest {
  /// Roots holding the raw XML, read through the VFS so an installation imports like a loose tree.
  pub roots: XrayRoots,
  /// The language every entry this run reads is filed under. Never `all`.
  pub language: String,
  /// Where inside those roots to look, or nothing to let the run resolve it.
  pub prefix: Option<String>,
  /// Directory the JSON sources are written to, which may already hold some.
  pub output_dir: PathBuf,
  /// Restrict the run to one table, by the file name it has in the scope.
  pub file: Option<String>,
  /// Let incoming text replace existing text that differs, instead of keeping what is there.
  pub is_overwrite: bool,
  /// Do everything except write, so a caller can see what a run would change.
  pub is_dry_run: bool,
}

/// What an import run reports back to the desktop surface.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationParseSummary {
  /// Whether the run read every table or was stopped between them.
  pub outcome: xrf_job::JobOutcome,
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
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationParseRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationParseSummary> {
  let language: TranslationLanguage =
    TranslationLanguage::from_str_single(&request.language).map_err(error_to_string)?;

  log::info!(
    "Parsing translations: {} root(s), '{language}', into {}{}",
    request.roots.roots.len(),
    format_path(&request.output_dir),
    if request.is_dry_run { " (dry run)" } else { "" }
  );

  // The request travels whole rather than as a hand-picked subset, so a window that adopts this job after a reload can
  // say what it was actually asked to do rather than a summary somebody chose in advance.
  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, PARSE_JOB_KIND)
      .with_lease_keys(vec![to_output_lease_key(&request.output_dir)])
      .with_request(&request)
      .with_progress(progress),
  )?;

  let TranslationParseRequest {
    roots,
    prefix,
    output_dir,
    file,
    is_overwrite,
    is_dry_run,
    ..
  } = request;

  let options: ProjectParseOptions = ProjectParseOptions {
    job: job.clone(),
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
  // Concluded with the summary rather than the crate's own result, because that is what this command answers: a window
  // that adopts this job after a reload reads the registry's copy and has to find the shape it would have been given.
  let outcome: TauriResult<TranslationParseSummary> =
    tauri::async_runtime::spawn_blocking(move || xrf_translation::parse_translations(&options))
      .await
      .map_err(|error| format!("Translation import did not finish: {error}"))?
      .map_err(error_to_string)
      .map(|result: ProjectParseResult| TranslationParseSummary {
        language: result.language.clone(),
        outcome: result.outcome,
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
      });

  registration.conclude_with(&outcome, job.is_cancelled());

  if let Ok(summary) = &outcome {
    log::info!(
      "Parsed {} translation file(s), {} created, {} updated, {} finding(s)",
      summary.census.files_read,
      summary.census.files_created,
      summary.census.files_updated,
      summary.findings.len()
    );
  }

  outcome
}
