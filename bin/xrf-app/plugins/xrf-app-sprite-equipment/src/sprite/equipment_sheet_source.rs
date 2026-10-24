use serde::{Deserialize, Serialize};

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
