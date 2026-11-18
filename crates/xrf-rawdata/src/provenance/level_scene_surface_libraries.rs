use serde::{Deserialize, Serialize};

use crate::provenance::level_scene_source::LevelSceneSource;

/// The libraries a scene's surfaces were dressed from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSurfaceLibraries {
  pub game_materials: LevelSceneSource,
  pub compiler_shaders: LevelSceneSource,
  /// The object library roots whose pairings broke ties, as given; a file there that could not be used is among the
  /// scene's refusals.
  pub object_roots: Vec<String>,
}
