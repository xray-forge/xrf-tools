use crate::feature::image::exposure::reading::exposure_metering::ExposureMetering;
use crate::feature::image::exposure::reading::pre_exposure::PreExposure;

/// What a view's exposure decided for its frame, prepared before every other unit so each reads this frame's choice:
/// whether the passes read the adapted scale, the scale the frame's light is written at and the last frame's, and what
/// the physical light's histogram last metered, which the game's pictures and the display transforms are shown by.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExposureDecisions {
  pub is_adapting: bool,
  pub pre_exposure: PreExposure,
  pub metering: ExposureMetering,
}
