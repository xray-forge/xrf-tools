use serde::Serialize;

use crate::plugins::levels::state::selection::level_spawn_object::LevelSpawnObject;

/// The spawned objects the viewer draws, and the visuals they stand as, each named once.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObjectsDescription {
  pub visuals: Vec<String>,
  pub objects: Vec<LevelSpawnObject>,
}
