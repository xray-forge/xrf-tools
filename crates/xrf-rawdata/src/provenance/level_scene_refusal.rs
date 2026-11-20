use serde::{Deserialize, Serialize};

/// What the decompile read but left out of the scene, with why.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneRefusal {
  /// The compiled file holding it, such as `level.spawn`, or an object library's file that could not be used.
  pub file: String,
  /// Its name there, such as an entity's, or the library file's name.
  pub object: String,
  pub reason: String,
}
