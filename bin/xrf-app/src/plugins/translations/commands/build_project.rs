use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use xrf_translation::{
  ProjectBuildLanguageSummary, ProjectBuildOptions, ProjectBuildResult, TranslationLanguage, build_roots,
};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// What a build reports back to the desktop surface.
///
/// A row per language rather than the 272 files behind a full run, which is the natural grain of a
/// build whose job is one string table per language.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationBuildSummary {
  /// The language built, or `all`.
  pub language: String,
  /// Sources read.
  pub sources: u32,
  /// String tables written, across every language.
  pub files: u32,
  /// Where they were written, so the result can offer to open it.
  pub output_dir: String,
  pub languages: Vec<ProjectBuildLanguageSummary>,
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
  roots: XrayRoots,
  prefix: Option<String>,
  language: String,
  output_dir: PathBuf,
  is_sorted: bool,
) -> TauriResult<TranslationBuildSummary> {
  // `all` is accepted: compiling every language at once is the ordinary build.
  let language: TranslationLanguage = TranslationLanguage::from_str(&language)?;
  let output_label: String = output_dir.display().to_string();

  log::info!(
    "Building translations: {} root(s), '{language}', into {output_label}",
    roots.roots.len()
  );

  let options: ProjectBuildOptions = ProjectBuildOptions {
    is_sorted,
    output: xrf_output::OutputOptions::default(),
    output_dir,
    language,
  };

  // Off the async worker: a full build compiles every source into eight string tables and writes all
  // of them, which is not work an IPC executor should be holding.
  let result: ProjectBuildResult =
    tauri::async_runtime::spawn_blocking(move || build_roots(&roots, prefix.as_deref(), &options))
      .await
      .map_err(|error| format!("Translation build did not finish: {error}"))?
      .map_err(error_to_string)?;

  log::info!(
    "Built {} string table(s) from {} source(s)",
    result.files,
    result.sources
  );

  Ok(TranslationBuildSummary {
    language: language.to_string(),
    sources: result.sources,
    files: result.files,
    output_dir: output_label,
    languages: result.languages,
  })
}
