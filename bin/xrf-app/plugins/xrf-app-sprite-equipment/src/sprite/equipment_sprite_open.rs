use serde::{Deserialize, Serialize};
use xrf_app_core::assets::AssetSource;
use xrf_vfs::XrayRoots;

use crate::EquipmentSheetSource;

/// Everything one opening of a sheet was asked to read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSpriteOpen {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  pub sheet: EquipmentSheetSource,
  /// The `system.ltx` naming what sits on the sheet, a file or an entry point of the roots, or nothing to open it
  /// unannotated.
  pub config: Option<AssetSource>,
  /// Whether to resolve that configuration with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
}

impl EquipmentSpriteOpen {
  /// Whether anything this open reads lies in the roots rather than in a file named by its path.
  pub fn reads_roots(&self) -> bool {
    matches!(self.sheet, EquipmentSheetSource::Asset { .. }) || matches!(self.config, Some(AssetSource::Asset { .. }))
  }
}
