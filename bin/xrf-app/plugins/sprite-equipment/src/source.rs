use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

/// Where an equipment sheet is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum EquipmentSheetSource {
  /// A loose `.dds` on disk, named by its filesystem path.
  File { path: String },
  /// A sheet of the roots, named by its engine reference such as `ui\ui_icon_equipment`.
  Asset { reference: String },
}

impl EquipmentSheetSource {
  /// What to call this sheet on screen.
  pub fn label(&self) -> &str {
    match self {
      Self::File { path } => path,
      Self::Asset { reference } => reference,
    }
  }
}

/// Where the configuration that annotates a sheet is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum EquipmentConfigSource {
  /// A `system.ltx` on disk, named by its filesystem path.
  File { path: String },
  /// An entry point of the roots, named by its logical path such as `configs\system.ltx`.
  Asset { logical_path: String },
}

impl EquipmentConfigSource {
  /// What to call this configuration on screen.
  pub fn label(&self) -> &str {
    match self {
      Self::File { path } => path,
      Self::Asset { logical_path } => logical_path,
    }
  }
}

/// Everything one opening of a sheet was asked to read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EquipmentSpriteOpen {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  pub sheet: EquipmentSheetSource,
  /// The configuration naming what sits on the sheet, or nothing to open it unannotated.
  pub config: Option<EquipmentConfigSource>,
  /// Whether to resolve that configuration with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
}
