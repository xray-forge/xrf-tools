use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_material::XrayTextureScope;
use xrf_vfs::XrayLogicalPath;

use crate::plugins::levels::state::LEVELS_DIRECTORY;

/// Where a compiled level is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelSource {
  /// A compiled level directory on disk, named by its filesystem path.
  Directory { path: String },
  /// A level of the mounted roots, named by its engine identity.
  Asset { logical_path: String },
}

impl LevelSource {
  pub fn get_label(&self) -> &str {
    match self {
      Self::Directory { path } => path,
      Self::Asset { logical_path } => logical_path,
    }
  }

  /// Returns the level's filesystem path when its source provides one.
  pub fn get_physical_path(&self) -> Option<&Path> {
    match self {
      Self::Directory { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }

  /// The level's engine identity: the directory its own files are addressed under.
  pub fn get_logical_directory(&self) -> Option<XrayLogicalPath> {
    match self {
      Self::Asset { logical_path } => XrayLogicalPath::new(logical_path).ok(),
      Self::Directory { path } => Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
          XrayLogicalPath::new(LEVELS_DIRECTORY)
            .and_then(|levels| levels.join(name))
            .ok()
        }),
    }
  }

  /// Where its surfaces' textures and descriptors are read: beside the level, which the engine mounts as `$level$`,
  /// then the shared tree; the shared tree alone for a level with no engine identity.
  pub fn get_texture_scope(&self) -> XrayTextureScope {
    self
      .get_logical_directory()
      .map_or_else(XrayTextureScope::shared, XrayTextureScope::of_level)
  }

  /// The level's name, the last component of its engine identity, which the game graph knows it by.
  pub fn get_name(&self) -> Option<String> {
    self.get_logical_directory().map(|path| path.file_name().to_owned())
  }
}

#[cfg(test)]
mod tests {
  use xrf_vfs::XrayLogicalPath;

  use crate::plugins::levels::state::LevelSource;

  #[test]
  fn names_an_asset_level_by_the_path_it_was_opened_as() {
    assert_eq!(
      LevelSource::Asset {
        logical_path: String::from("levels\\k00_marsh"),
      }
      .get_logical_directory()
      .as_ref()
      .map(XrayLogicalPath::as_str),
      Some("levels\\k00_marsh")
    );
  }

  #[test]
  fn names_a_directory_level_by_its_own_name_below_levels() {
    // A loose directory still has an engine identity, which is what its lightmaps are addressed under.
    assert_eq!(
      LevelSource::Directory {
        path: String::from("C:\\game\\gamedata\\levels\\l01_escape"),
      }
      .get_logical_directory()
      .as_ref()
      .map(XrayLogicalPath::as_str),
      Some("levels\\l01_escape")
    );
  }

  #[test]
  fn names_a_level_by_the_last_component_of_its_identity_in_lower_case() {
    assert_eq!(
      LevelSource::Asset {
        logical_path: String::from("levels\\K00_Marsh"),
      }
      .get_name()
      .as_deref(),
      Some("k00_marsh")
    );
  }

  #[test]
  fn names_nothing_for_a_directory_with_no_final_component() {
    assert_eq!(
      LevelSource::Directory { path: String::from("") }.get_logical_directory(),
      None
    );
  }
}
