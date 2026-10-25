use std::collections::HashMap;

use serde::Serialize;
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::XraySearch;

use crate::report::LevelReport;

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

impl LevelSpawnRelease {
  /// The configs listing the objects a new game releases, and the section of each holding their names: the objects it
  /// removes outright, and the items it replaces with ones of its own.
  const RELEASES: [(&'static str, &'static str, Self); 2] = [
    (
      "configs/plugins/new_game_setup.ltx",
      "remove_objects",
      Self::RemoveObjects,
    ),
    (
      "configs/items/settings/dynamic_item_spawn.ltx",
      "replace_items",
      Self::ReplaceItems,
    ),
  ];

  /// The spawned objects a new game releases by name, and why; none of a list the game does not have, or cannot be
  /// read. Anomaly's `game_setup.script` takes them off the levels before the actor first stands on one, so a level
  /// viewer marks them and draws them only when asked.
  pub fn read_new_game(search: &XraySearch) -> HashMap<String, Self> {
    let mut released: HashMap<String, Self> = HashMap::new();

    for (file, section, release) in Self::RELEASES {
      match Self::read_released(search, file, section) {
        Ok(names) => {
          for name in names {
            released.entry(name).or_insert(release);
          }
        }
        Err(error) => LevelReport::unreadable_releases(file, &error),
      }
    }

    released
  }

  /// The names one config's section lists, as keys; none where the config or the section is missing.
  pub fn list_released(text: &str, section: &str) -> XrfResult<Vec<String>> {
    Ok(
      Ltx::read_from_str(text)?
        .section(section)
        .map(|it| it.iter().map(|(name, _)| name.to_owned()).collect())
        .unwrap_or_default(),
    )
  }

  // todo: Resolve both configs under the level's dialect, as Anomaly's `ini_file` applies DLTX `mod_` files to them.
  fn read_released(search: &XraySearch, file: &str, section: &str) -> XrfResult<Vec<String>> {
    let Some(asset) = search.find(file)? else {
      return Ok(Vec::new());
    };
    let bytes: Vec<u8> = search.read_asset(&asset)?;

    Self::list_released(&decode_bytes_to_string(&bytes, new_windows1251_encoder())?, section)
  }
}
