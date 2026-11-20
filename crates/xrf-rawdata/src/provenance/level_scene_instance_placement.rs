use serde::{Deserialize, Serialize};

/// One placement of a library object found in the static geometry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneInstancePlacement {
  pub reference: String,
  /// The scene object placing it.
  pub object: String,
  /// How much of its rendered area the level drew as its own triangles, from 0 to 1, and how far its placed corners
  /// stood from the level's they were matched to, metres.
  pub coverage: f64,
  pub error: f64,
  /// The level triangles the placement took out of the shells.
  pub triangles: usize,
}
