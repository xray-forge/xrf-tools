use std::collections::HashMap;

use serde::Serialize;
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_ltx_project::LtxProject;
use xrf_vfs::XrayLogicalPath;

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
  /// The configs listing the objects a new game releases, below `configs` as `game_setup.script` names them to
  /// `ini_file`, and the section of each holding their names: the objects it removes outright, and the items it
  /// replaces with ones of its own.
  const RELEASES: [(&'static str, &'static str, Self); 2] = [
    ("plugins\\new_game_setup.ltx", "remove_objects", Self::RemoveObjects),
    (
      "items\\settings\\dynamic_item_spawn.ltx",
      "replace_items",
      Self::ReplaceItems,
    ),
  ];

  /// The spawned objects a new game releases by name, and why; none of a list the game does not have, or cannot be
  /// read. Anomaly's `game_setup.script` takes them off the levels before the actor first stands on one, so a level
  /// viewer marks them and draws them only when asked.
  ///
  /// Each list is resolved under `project`'s dialect, as `ini_file` reads it, so a DLTX `mod_` file changing it counts.
  pub fn read_new_game(project: &LtxProject) -> HashMap<String, Self> {
    let mut released: HashMap<String, Self> = HashMap::new();

    for (file, section, release) in Self::RELEASES {
      match Self::read_released(project, file, section) {
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

  /// The names one section of a config lists, as keys; none where the section is missing.
  pub fn list_released(ltx: &Ltx, section: &str) -> Vec<String> {
    ltx
      .section(section)
      .map(|it| it.iter().map(|(name, _)| name.to_owned()).collect())
      .unwrap_or_default()
  }

  /// The names one list holds once the dialect has applied what patches it; none where the game has no such list.
  fn read_released(project: &LtxProject, file: &str, section: &str) -> XrfResult<Vec<String>> {
    let path: XrayLogicalPath = project.config_path(file)?;

    if project.search().find(path.as_str())?.is_none() {
      return Ok(Vec::new());
    }

    Ok(Self::list_released(&project.read_resolution(&path)?.ltx, section))
  }
}
