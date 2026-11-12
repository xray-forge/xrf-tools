use serde::Serialize;

use crate::level::level_geometry_conversion::LevelGeometryCensus;
use crate::provenance::level_scene_provenance::{LevelSceneInference, LevelSceneRefusal};

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
  /// The static geometry, counted, where the level held `level.geom`; the sidecar lists each object.
  pub geometry: Option<LevelDecompileGeometry>,
}

/// One tool of the scene a decompile wrote, and what its part holds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileReportTool {
  pub name: String,
  /// The objects its text part holds; none for a binary part.
  pub objects: Option<usize>,
}

/// The static geometry a decompile wrote, counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileGeometry {
  /// The objects written, one per sector drawing static geometry.
  pub objects: usize,
  #[serde(flatten)]
  pub census: LevelGeometryCensus,
  /// How the surfaces were dressed, where they were inferred; the sidecar lists each compiler shader chosen.
  pub surfaces: Option<LevelDecompileSurfaces>,
}

/// The surfaces a decompile inferred, counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileSurfaces {
  /// Triangles on a collision face of the same three points.
  pub exact_triangles: usize,
  /// Triangles a short ray found a collision face beneath.
  pub traced_triangles: usize,
  /// Triangles on no collision face.
  pub uncollided_triangles: usize,
  /// Compiler shaders chosen, one per engine shader and kind of evidence.
  pub compiler_shaders: usize,
  /// Collision materials the game material library does not define.
  pub unknown_game_materials: usize,
}

/// The root spawn a decompile read, counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDecompileRootSpawn {
  pub file: String,
  /// Its objects on the level, which the scene's entities came from; none where it does not hold the level.
  pub objects: usize,
  pub disagreements: usize,
  pub update_packets: usize,
}
