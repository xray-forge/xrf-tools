use serde::{Deserialize, Serialize};

use crate::contract::render_sun_shafts_quality::RenderSunShaftsQuality;

/// How the sun's light shafts are drawn, beside the view's switch for them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSunShafts {
  pub quality: RenderSunShaftsQuality,
  /// `r2_sunshafts_min`, from zero to a half: the floor the keyframes' density is lifted from; zero draws it as it is.
  pub minimum: f32,
}
