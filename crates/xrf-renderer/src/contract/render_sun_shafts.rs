use serde::{Deserialize, Serialize};
use xrf_engine_target::XrayEngine;

use crate::contract::render_sun_shafts_quality::RenderSunShaftsQuality;

/// How the sun's light shafts are drawn, beside the view's switch for them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSunShafts {
  pub quality: RenderSunShaftsQuality,
  /// `r2_sunshafts_min`, from zero to a half: the floor the keyframes' density is lifted from; zero draws it as it is.
  pub minimum: f32,
  /// Steps along a ray in place of the quality's, up to `RenderSunShafts::MAX_STEPS`; none steps as the quality does.
  pub steps: Option<u32>,
}

impl RenderSunShafts {
  /// The most steps a ray is asked to take.
  pub const MAX_STEPS: u32 = 64;

  /// Steps along a ray `accum_volumetric_sun` takes: those asked for, else the quality's on the engine drawn.
  pub fn get_steps(&self, engine: XrayEngine) -> u32 {
    self.steps.map_or_else(
      || self.quality.get_steps(engine),
      |steps| steps.clamp(1, Self::MAX_STEPS),
    )
  }
}
