use serde::{Deserialize, Serialize};

/// One compiled file a scene was decompiled from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSource {
  /// Its name in the level's directory, such as `level.ps_static`.
  pub file: String,
  /// Its SHA-256, lowercase hex.
  pub sha256: String,
}
