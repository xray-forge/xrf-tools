use std::path::PathBuf;

use serde::Serialize;
use xrf_vfs::XrayAsset;

/// Where an opened sheet turned out to be, and whether anything can write there.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSheetLocation {
  /// The sheet as the roots located it, or nothing for a file opened by path outside any tree.
  pub asset: Option<XrayAsset>,
  /// Absolute path of the sheet, or nothing when it lives inside an archive volume.
  pub path: Option<String>,
  /// Where a write to the sheet would land.
  pub write_target: Option<String>,
}

impl EquipmentSheetLocation {
  /// Locates a sheet opened by path, which is its own write target: nothing shadows it and nothing has to.
  pub fn of_file(path: &str) -> Self {
    Self {
      asset: None,
      path: Some(path.into()),
      write_target: Some(path.into()),
    }
  }

  /// Locates a sheet the roots resolved.
  pub fn of_asset(asset: XrayAsset) -> Self {
    Self {
      path: Self::to_display_path(asset.to_physical_path()),
      write_target: Self::to_display_path(asset.to_writable_path().ok()),
      asset: Some(asset),
    }
  }

  /// Spells a path the way the rest of the editor shows one.
  fn to_display_path(path: Option<PathBuf>) -> Option<String> {
    path.map(|path| path.display().to_string())
  }
}
