use std::path::PathBuf;

use crate::spawn::spawn_root_options::SpawnRootOptions;

/// A root spawn to build from a gamedata root's levels into its `spawns/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnRootRequest {
  /// The root holding `configs/game_levels.ltx`, the levels' directories under `levels/`, and `spawns/`.
  pub gamedata: PathBuf,
  /// The levels to build, by directory name; none builds every level `[levels]` lists whose directory holds
  /// `level.ai`, `level.spawn` and `level.game`.
  pub levels: Option<Vec<String>>,
  /// The spawn's name, written as `spawns/<name>.spawn`.
  pub name: String,
  pub options: SpawnRootOptions,
  /// Write over `spawns/all.spawn`, the game's own, where it is there.
  pub is_forced: bool,
}
