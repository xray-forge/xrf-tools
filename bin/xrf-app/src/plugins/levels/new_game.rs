//! What a new game takes off the levels before the actor first stands on one: Anomaly's `game_setup.script` releases
//! every spawned object two of its configs name, so a level viewer leaves them out too.

use std::collections::HashSet;

use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::XrayProbe;

use crate::plugins::levels::report::report_unreadable_releases;

/// The configs listing the objects a new game releases, and the section of each holding their names: the objects it
/// removes outright, and the items it replaces with ones of its own.
const RELEASES: [(&str, &str); 2] = [
  ("configs/plugins/new_game_setup.ltx", "remove_objects"),
  ("configs/items/settings/dynamic_item_spawn.ltx", "replace_items"),
];

/// Names of the spawned objects a new game releases; none of a list the game does not have, or cannot be read.
pub fn read_new_game_releases(probe: &XrayProbe) -> HashSet<String> {
  let mut released: HashSet<String> = HashSet::new();

  for (file, section) in RELEASES {
    match read_released(probe, file, section) {
      Ok(names) => released.extend(names),
      Err(error) => report_unreadable_releases(file, &error),
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

fn read_released(probe: &XrayProbe, file: &str, section: &str) -> XrfResult<Vec<String>> {
  let resolution = probe.find(file)?;
  let Some(asset) = resolution.get_asset() else {
    return Ok(Vec::new());
  };
  let bytes: Vec<u8> = probe.read_asset_bytes(asset)?;

  list_released(&decode_bytes_to_string(&bytes, new_windows1251_encoder())?, section)
}
