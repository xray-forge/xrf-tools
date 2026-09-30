use serde::Serialize;

use crate::plugins::levels::state::selection::level_spawn_model_description::LevelSpawnModelDescription;
use crate::plugins::levels::state::selection::level_spawn_model_failure::LevelSpawnModelFailure;
use crate::plugins::levels::state::selection::level_spawn_object_hemi::LevelSpawnObjectHemi;

/// The models of one batch of spawned visuals: each one read, why each other one could not be, and how the level
/// lights each object standing as one read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnModelsDescription {
  pub models: Vec<LevelSpawnModelDescription>,
  pub failures: Vec<LevelSpawnModelFailure>,
  /// None where the level's collision form cannot be read, which lights every object as if under the open sky.
  pub hemi: Vec<LevelSpawnObjectHemi>,
}
