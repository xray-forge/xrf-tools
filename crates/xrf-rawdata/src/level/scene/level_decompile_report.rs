use serde::Serialize;

use crate::level::scene::level_decompile_geometry::LevelDecompileGeometry;
use crate::level::scene::level_decompile_report_tool::LevelDecompileReportTool;
use crate::level::scene::level_decompile_root_spawn::LevelDecompileRootSpawn;
use crate::provenance::level_scene_inference::LevelSceneInference;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// What a decompile into a scene on disk wrote ([`crate::LevelSceneDecompiler::decompile_to_path`]), as the front-ends
/// report it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileReport {
  pub source: String,
  pub destination: String,
  pub sidecar: String,
  /// The compiled files the scene came from.
  pub sources: Vec<String>,
  pub tools: Vec<LevelDecompileReportTool>,
  pub inferred: Vec<LevelSceneInference>,
  /// What the scene leaves out, such as an entity of a class the decompile cannot type.
  pub refused: Vec<LevelSceneRefusal>,
  /// The root spawn read, where one was; the sidecar lists what is counted here.
  pub root_spawn: Option<LevelDecompileRootSpawn>,
  /// The objects root the scene's objects were written under.
  pub objects_root: String,
  /// The game textures root the objects' textures were written under, and how many.
  pub textures_root: String,
  pub textures: usize,
  /// The parts and objects of the level's own an earlier decompile wrote and this forced one does not, removed.
  pub removed: Vec<String>,
  /// The static geometry, counted, where the level held `level.geom`; the sidecar lists each object.
  pub geometry: Option<LevelDecompileGeometry>,
}
