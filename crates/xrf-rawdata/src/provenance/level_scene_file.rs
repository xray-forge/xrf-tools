use serde::{Deserialize, Serialize};

/// One file the decompile wrote, and the compiled files it came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneFile {
  /// Its path relative to the scene's main file's directory, such as `labx8/ps.part`.
  pub written: String,
  pub sources: Vec<String>,
}
