use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::root::spawn_root_options::SpawnRootOptions;

/// A root spawn to build from a gamedata root's level list and the levels' directories, written as `<name>.spawn`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnRootRequest {
  /// The root holding `configs/game_levels.ltx`, and by default the levels and the spawns.
  pub gamedata: PathBuf,
  /// The directory holding the levels' directories; none for `levels/` under `gamedata`.
  pub levels_root: Option<PathBuf>,
  /// The levels to build, by directory name; none builds every level `[levels]` lists whose directory holds
  /// `level.ai`, `level.spawn` and `level.game`.
  pub levels: Option<Vec<String>>,
  /// The directory the spawn is written into; none for `spawns/` under `gamedata`.
  pub destination: Option<PathBuf>,
  /// The spawn's name, written as `<name>.spawn`.
  pub name: String,
  pub options: SpawnRootOptions,
  /// Write over `all.spawn`, the game's own, where it is there.
  pub is_forced: bool,
}

impl SpawnRootRequest {
  /// Where a gamedata root keeps its levels' directories.
  pub const LEVELS_DIRECTORY: &'static str = "levels";
  /// Where a gamedata root keeps its spawns.
  pub const SPAWNS_DIRECTORY: &'static str = "spawns";

  /// The directory holding the levels' directories: the one named, else `levels/` under the gamedata root.
  pub fn get_levels_root(&self) -> PathBuf {
    self
      .levels_root
      .clone()
      .unwrap_or_else(|| self.gamedata.join(Self::LEVELS_DIRECTORY))
  }

  /// The spawn file written: `<name>.spawn` in the directory named, else in `spawns/` under the gamedata root.
  pub fn get_spawn_path(&self) -> PathBuf {
    self
      .destination
      .as_deref()
      .map_or_else(|| self.gamedata.join(Self::SPAWNS_DIRECTORY), Path::to_path_buf)
      .join(format!("{}.spawn", self.name))
  }
}
