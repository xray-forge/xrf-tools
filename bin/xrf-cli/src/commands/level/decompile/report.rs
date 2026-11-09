use serde::Serialize;
use xrf_rawdata::{LevelGeometryCensus, LevelSceneInference, LevelSceneRefusal};

/// One tool of the scene `level decompile` wrote, and what its part holds.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelDecompileTool {
  pub(crate) name: String,
  /// The objects its text part holds; none for a binary part.
  pub(crate) objects: Option<usize>,
}

/// What `level decompile` wrote, as its JSON report says it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelDecompileReport {
  pub(crate) source: String,
  pub(crate) destination: String,
  pub(crate) sidecar: String,
  /// The compiled files the scene came from.
  pub(crate) sources: Vec<String>,
  pub(crate) tools: Vec<LevelDecompileTool>,
  pub(crate) inferred: Vec<LevelSceneInference>,
  /// What the scene leaves out, such as an entity of a class the decompile cannot type.
  pub(crate) refused: Vec<LevelSceneRefusal>,
  /// The root spawn read, where one was; the sidecar lists what is counted here.
  pub(crate) root_spawn: Option<LevelDecompileRootSpawn>,
  /// The objects root the scene's objects were written under.
  pub(crate) objects_root: String,
  /// The static geometry, counted, where the level held `level.geom`; the sidecar lists each object.
  pub(crate) geometry: Option<LevelDecompileGeometry>,
}

/// The static geometry `level decompile` wrote, counted.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelDecompileGeometry {
  /// The objects written, one per sector drawing static geometry.
  pub(crate) objects: usize,
  #[serde(flatten)]
  pub(crate) census: LevelGeometryCensus,
  /// How the surfaces were dressed, where they were inferred; the sidecar lists each compiler shader chosen.
  pub(crate) surfaces: Option<LevelDecompileSurfaces>,
}

/// The surfaces `level decompile` inferred, counted.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelDecompileSurfaces {
  /// Triangles on a collision face of the same three points.
  pub(crate) exact_triangles: usize,
  /// Triangles a short ray found a collision face beneath.
  pub(crate) traced_triangles: usize,
  /// Triangles on no collision face.
  pub(crate) uncollided_triangles: usize,
  /// Compiler shaders chosen, one per engine shader and kind of evidence.
  pub(crate) compiler_shaders: usize,
  /// Collision materials the game material library does not define.
  pub(crate) unknown_game_materials: usize,
}

/// The root spawn `level decompile` read, counted.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelDecompileRootSpawn {
  pub(crate) file: String,
  /// Its objects on the level, which the scene's entities came from; none where it does not hold the level.
  pub(crate) objects: usize,
  pub(crate) disagreements: usize,
  pub(crate) update_packets: usize,
}
