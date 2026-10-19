use serde::{Deserialize, Serialize};

/// How the sun's light shafts are drawn, beside the view's switch for them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSunShafts {
  /// Steps along a ray `accum_volumetric_sun` takes; the engines' `r2_sunshafts_quality` steps OpenXRay's 20, 20 and
  /// 40, Monolith's 15, 25 and 30.
  pub steps: u32,
  /// `r2_sunshafts_min`, from zero to a half: the floor the keyframes' density is lifted from; zero draws it as it is.
  pub minimum: f32,
}

impl Default for RenderSunShafts {
  /// Monolith's high quality, and no floor.
  fn default() -> Self {
    Self {
      steps: Self::STEPS,
      minimum: 0.0,
    }
  }
}

impl RenderSunShafts {
  /// The steps by default, and the least and most there may be.
  pub const STEPS: u32 = 30;
  pub const STEPS_RANGE: (u32, u32) = (8, 64);

  /// The steps along a ray, held to their range.
  pub fn get_steps(&self) -> u32 {
    self.steps.clamp(Self::STEPS_RANGE.0, Self::STEPS_RANGE.1)
  }
}
