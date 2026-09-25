use serde::Serialize;

use crate::plugins::levels::state::selection::level_spawn_model_description::LevelSpawnModelDescription;
use crate::plugins::levels::state::selection::level_spawn_placement::LevelSpawnPlacement;

/// The models a level's spawned objects are drawn as, and where each object stands.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnModelsDescription {
  pub models: Vec<LevelSpawnModelDescription>,
  pub placements: Vec<LevelSpawnPlacement>,
}
