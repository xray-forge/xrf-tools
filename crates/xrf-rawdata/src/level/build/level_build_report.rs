use serde::Serialize;

use crate::level::build::level_build_census::LevelBuildCensus;
use crate::level::build::level_exported_file::LevelExportedFile;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// What [`crate::LevelBuildExporter::build_to_directory`] wrote, for the front-ends to report.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildReport {
  pub source: String,
  pub destination: String,
  /// Each file written, its size and what it replaced.
  pub files: Vec<LevelExportedFile>,
  /// Files an earlier build wrote that this one did not, removed, by path in the directory.
  pub removed: Vec<String>,
  /// What `build.prj` took from the scene, where the build wrote one.
  pub geometry: Option<LevelBuildCensus>,
  /// The parts the build made nothing of, and why.
  pub refused: Vec<LevelSceneRefusal>,
}
