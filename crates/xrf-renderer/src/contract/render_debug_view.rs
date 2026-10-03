use serde::{Deserialize, Serialize};

/// Which picture a viewport shows: its finished frame, or one of the targets the frame was built from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDebugView {
  /// The finished frame.
  #[default]
  Final,
  /// The G-buffer's albedo, raw.
  Albedo,
  /// The gloss the albedo target carries in its alpha.
  Gloss,
  /// The view space normal, remapped to colour.
  Normal,
  /// The baked hemisphere occlusion.
  Hemi,
  /// The baked sun occlusion.
  Sun,
  /// The lighting model slice, `(class + 0.5) / 4`.
  Material,
  /// View distance, logarithmic: near dark, far light.
  Depth,
  /// What the sun and the lights accumulated.
  Light,
  /// The screen's occlusion, white where it is off.
  AmbientOcclusion,
}

impl RenderDebugView {
  /// The number the present shader tells it by.
  pub fn get_index(self) -> u32 {
    self as u32
  }
}
