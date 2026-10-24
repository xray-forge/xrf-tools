use serde::Serialize;

/// What the viewer shows of one spawned object beyond where it stands: what the game knows it by and what it carries.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObjectDetails {
  /// Its ALife id in the spawn.
  pub id: u16,
  /// The game graph vertex it stands at, which places it among the levels; the engine's invalid `u16::MAX` for an
  /// object without the abstract part that carries it.
  pub game_vertex_id: u16,
  /// The level graph vertex it stands at, which places it on the level's AI map; the engine's invalid `u32::MAX` where
  /// it has none.
  pub level_vertex_id: u32,
  /// Its `custom_data`, the logic and settings a script reads, empty where it has none.
  pub custom_data: String,
}
