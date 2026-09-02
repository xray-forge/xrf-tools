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

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::translations::lease::{BUILD_JOB_KIND, to_output_lease_key};

/// What a build was asked to do.
///
/// One argument rather than five, because a Tauri command's parameters are its wire signature and five of them plus a
/// job's own two is more than a reader can hold. It is also exactly what the registry retains, so a window adopting
/// this run after a reload sees the request rather than a summary of it.
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
///
/// A row per language rather than the 272 files behind a full run, which is the natural grain of a
/// build whose job is one string table per language.
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
///
/// `roots` names where the sources are read from, through the VFS, so a tree layered over an
/// installation compiles what the engine would actually load. `outputDir` is a plain host directory,
/// because a string table is a file and a `.db` volume has nowhere to put one.
///
/// Refuses an output directory inside any of the source roots before writing anything.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "build_project"))]
#[tauri::command(rename = "build_project")]
pub async fn translations_build_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationBuildRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationBuildSummary> {
  // `all` is accepted: compiling every language at once is the ordinary build.
  let language: TranslationLanguage = TranslationLanguage::from_str(&request.language)?;

  log::info!(
    "Building translations: {} root(s), '{language}', into {}",
    request.roots.roots.len(),
    format_path(&request.output_dir)
  );

  // The request travels whole rather than as a hand-picked subset, so a window that adopts this job after a reload can
  // say what it was actually asked to do rather than a summary somebody chose in advance.
  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, BUILD_JOB_KIND)
      .with_lease_keys(vec![to_output_lease_key(&request.output_dir)])
      .with_request(&request)
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
    job: job.clone(),
    is_sorted,
    output: xrf_output::OutputOptions::default(),
    output_dir,
    language,
  };

  // Off the async worker: a full build compiles every source into eight string tables and writes all
  // of them, which is not work an IPC executor should be holding.
  // Concluded with the summary rather than the crate's own result, because that is what this command answers: a window
  // that adopts this job after a reload reads the registry's copy and has to find the shape it would have been given.
  let outcome: TauriResult<TranslationBuildSummary> = execution
    .run_blocking("Translation build", move || {
      TranslationBuilder::build_roots(&roots, prefix.as_deref(), &options)
    })
    .await?
    .map_err(error_to_string)
    .map(|result: TranslationBuildResult| TranslationBuildSummary {
      language: language.to_string(),
      outcome: result.outcome,
      sources: result.sources,
      files: result.files,
      languages: result.languages,
    });

  registration.conclude_with(&outcome, job.is_cancelled());

  if let Ok(summary) = &outcome {
    log::info!(
      "Built {} string table(s) from {} source(s)",
      summary.files,
      summary.sources
    );
  }

  outcome
}
