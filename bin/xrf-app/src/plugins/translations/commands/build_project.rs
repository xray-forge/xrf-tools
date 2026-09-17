use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::{
  TranslationBuildLanguageSummary, TranslationBuildOptions, TranslationBuildResult, TranslationBuilder,
  TranslationLanguage,
};
use xrf_utils::format_path;
use xrf_vfs::XrayRoots;

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;

/// What a build was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationBuildRequest {
  /// Where the sources are read from, through the VFS.
  pub roots: XrayRoots,
  /// Where inside those roots to look, or nothing for the whole set.
  pub prefix: Option<String>,
  /// The language to build, or `all`.
  pub language: String,
  /// Directory the string tables are written into, which is always a host path.
  pub output_dir: PathBuf,
  /// Whether to sort entries within each table.
  pub is_sorted: bool,
}

/// What a build reports back to the desktop surface.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationBuildSummary {
  /// Whether the run compiled every source or was stopped between them.
  pub outcome: xrf_job::JobOutcome,
  /// The language built, or `all`.
  pub language: String,
  /// Sources read.
  pub sources: u32,
  /// String tables written, across every language.
  pub files: u32,
  pub languages: Vec<TranslationBuildLanguageSummary>,
}

/// Compile translation sources into per-language string tables.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "build_project"))]
#[tauri::command(rename = "build_project")]
pub async fn translations_build_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationBuildRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationBuildSummary> {
  let start: JobStart = JobStart::new(job_id, JobKind::TranslationsBuild).with_request(&request);

  // `all` is accepted: compiling every language at once is the ordinary build.
  let language: TranslationLanguage = TranslationLanguage::from_str(&request.language)?;

  log::info!(
    "Building translations: {} root(s), '{language}', into {}",
    request.roots.roots.len(),
    format_path(&request.output_dir)
  );

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::TranslationsBuild.as_str())
      .with_resources(vec![JobResource::tree(&request.output_dir)])
      .with_progress(progress),
  )?;

  let TranslationBuildRequest {
    roots,
    prefix,
    output_dir,
    is_sorted,
    ..
  } = request;

  let options: TranslationBuildOptions = TranslationBuildOptions {
    job,
    is_sorted,
    output: xrf_output::OutputOptions::default(),
    output_dir,
    language,
  };

  let outcome: TauriResult<TranslationBuildSummary> = run_job(
    &execution,
    "Translation build",
    registration,
    move || {
      TranslationBuilder::build_roots(&roots, prefix.as_deref(), &options).map(|result: TranslationBuildResult| {
        TranslationBuildSummary {
          language: language.to_string(),
          outcome: result.outcome,
          sources: result.sources,
          files: result.files,
          languages: result.languages,
        }
      })
    },
    |summary| summary.outcome,
  )
  .await;

  if let Ok(summary) = &outcome {
    log::info!(
      "Built {} string table(s) from {} source(s)",
      summary.files,
      summary.sources
    );
  }

  outcome
}
