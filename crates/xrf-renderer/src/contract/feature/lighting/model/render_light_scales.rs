use serde::{Deserialize, Serialize};

/// How a game's console scales the weather's light (`r2_sun_lumscale`, `r2_sun_lumscale_hemi`,
/// `r2_sun_lumscale_amb`): the sun's colour, and the hemisphere and the ambient the deferred frame is combined with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLightScales {
  pub sun: f32,
  pub hemi: f32,
  pub ambient: f32,
}

impl Default for RenderLightScales {
  /// The engine's own: nothing scaled.
  fn default() -> Self {
    Self {
      sun: 1.0,
      hemi: 1.0,
      ambient: 1.0,
    }
  }
}
