use serde::{Deserialize, Serialize};

/// What a Save as does with the original's bump map.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum TextureSaveAsBump {
  /// Copy the bump map files under the new name and point the copy's descriptor at them, so the copy stands alone.
  #[default]
  Own,
  /// Copy no bump map files; the copy keeps using the original's.
  Shared,
}
