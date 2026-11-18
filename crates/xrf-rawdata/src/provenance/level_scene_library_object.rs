use serde::{Deserialize, Serialize};

/// One library object a decompile copied beside the scene's objects.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneLibraryObject {
  /// Its reference, the path it is copied to under `$objects$` without the extension.
  pub reference: String,
  /// The file it was copied from, and its SHA-256, lowercase hex.
  pub source: String,
  pub sha256: String,
}
