use serde::Serialize;
use xrf_rawdata::{LevelSceneInference, LevelSceneRefusal};

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
