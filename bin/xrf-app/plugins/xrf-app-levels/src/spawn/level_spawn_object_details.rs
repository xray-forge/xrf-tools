use serde::Serialize;
use xrf_spawn::{AlifeObject, AlifeObjectAbstract};

/// What the viewer shows of one spawned object beyond where it stands: what the game knows it by and what it carries.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObjectDetails {
  /// Its ALife id in the spawn.
  pub id: u16,
  /// The game graph vertex it stands at, which places it among the levels; none for an object without the abstract
  /// part that carries it.
  pub game_vertex_id: Option<u16>,
  /// The level graph vertex it stands at, which places it on the level's AI map; none where it has none.
  pub level_vertex_id: Option<u32>,
  /// Its `custom_data`, the logic and settings a script reads, empty where it has none.
  pub custom_data: String,
}

impl LevelSpawnObjectDetails {
  /// What `object` carries, its vertices none where it has no abstract part rather than the engine's invalid values.
  pub fn of(object: &AlifeObject) -> Self {
    let base: Option<&AlifeObjectAbstract> = object.inherited.get_abstract();

    Self {
      id: object.id,
      game_vertex_id: base.map(|it| it.game_vertex_id),
      level_vertex_id: base.map(|it| it.level_vertex_id),
      custom_data: base.map(|it| it.custom_data.clone()).unwrap_or_default(),
    }
  }
}
