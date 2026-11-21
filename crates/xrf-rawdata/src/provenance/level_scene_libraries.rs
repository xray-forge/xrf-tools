use serde::{Deserialize, Serialize};

use crate::provenance::level_scene_source::LevelSceneSource;

/// The libraries a scene's static geometry was read with, each named and hashed, whatever dressed its surfaces: the
/// game materials that dressed them or told the placed library objects' ladders and breakables, and the compiler
/// shaders that dressed them or named the placed objects' surfaces. `None` where the decompile read none.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneLibraries {
  #[serde(default)]
  pub game_materials: Option<LevelSceneSource>,
  #[serde(default)]
  pub compiler_shaders: Option<LevelSceneSource>,
}
