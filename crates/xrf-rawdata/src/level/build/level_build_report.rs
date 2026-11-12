use serde::Serialize;

use crate::level::build::level_build_export::LevelBuildCensus;
use crate::level::level_exported_file::LevelExportedFile;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

/// What a build into a level directory wrote ([`crate::LevelBuildExporter::build_to_directory`]), as the front-ends
/// report it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildReport {
  pub source: String,
  pub destination: String,
  /// Each file written, its size and what it replaced.
  pub files: Vec<LevelExportedFile>,
  /// What `build.prj` took from the scene, where the build wrote one.
  pub geometry: Option<LevelBuildCensus>,
  /// The parts the build made nothing of, and why, as it reports them and goes on.
  pub refused: Vec<LevelSceneRefusal>,
}
