use serde::Serialize;
use xrf_rawdata::LevelSceneInference;

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
}
