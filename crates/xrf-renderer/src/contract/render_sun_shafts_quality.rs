use serde::{Deserialize, Serialize};
use xrf_engine_target::XrayEngine;

/// How finely the sun's light shafts step along a ray: the engines' `r2_sun_shafts` and `r2_sunshafts_quality` short
/// of off, which the view's switch is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderSunShaftsQuality {
  Low,
  Medium,
  #[default]
  High,
}

impl RenderSunShaftsQuality {
  /// Steps along a ray `accum_volumetric_sun` takes: OpenXRay's 20, 20 and 40, Monolith's 15, 25 and 30.
  pub const fn get_steps(self, engine: XrayEngine) -> u32 {
    match (engine, self) {
      (XrayEngine::Vanilla, Self::Low | Self::Medium) => 20,
      (XrayEngine::Vanilla, Self::High) => 40,
      (XrayEngine::Extended, Self::Low) => 15,
      (XrayEngine::Extended, Self::Medium) => 25,
      (XrayEngine::Extended, Self::High) => 30,
    }
  }
}
