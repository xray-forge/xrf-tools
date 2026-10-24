use serde::Serialize;

/// Why a new game releases a spawned object before the actor first stands on its level, as Anomaly's
/// `game_setup.script` does: the config and section naming it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelSpawnRelease {
  /// `configs/plugins/new_game_setup.ltx`, `[remove_objects]`: removed outright.
  RemoveObjects,
  /// `configs/items/settings/dynamic_item_spawn.ltx`, `[replace_items]`: replaced by an item the script places, which
  /// the viewer does not place.
  ReplaceItems,
}
