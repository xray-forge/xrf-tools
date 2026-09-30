use serde::Serialize;

use crate::plugins::levels::state::selection::level_spawn_model_description::LevelSpawnModelDescription;
use crate::plugins::levels::state::selection::level_spawn_model_failure::LevelSpawnModelFailure;

/// The models of one batch of spawned visuals: each one read, and why each other one could not be.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnModelsDescription {
  pub models: Vec<LevelSpawnModelDescription>,
  pub failures: Vec<LevelSpawnModelFailure>,
}
