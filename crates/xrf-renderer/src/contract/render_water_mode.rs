use serde::{Deserialize, Serialize};

/// Which water a viewport draws.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderWaterMode {
  /// The engine's own: `water.ps` and `waterd.ps`, reflecting the sky over the base.
  #[default]
  Engine,
  /// The enhanced water: what lies under it refracted, clouded with depth, the scene reflected, bordered softly.
  Enhanced,
}
