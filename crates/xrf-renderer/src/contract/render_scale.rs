use serde::{Deserialize, Serialize};
use xrf_dlss::DlssQuality;

/// How much smaller than the viewport the scene is drawn and then upscaled: the upscalers' quality modes, by the ratio of
/// the viewport's side to the drawing's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderScale {
  #[default]
  Native,
  /// 1.3: about three quarters of each side, between drawing whole and Quality.
  UltraQuality,
  /// 1.5: two thirds of each side.
  Quality,
  /// 1.7.
  Balanced,
  /// 2: half of each side.
  Performance,
  /// 3: a third of each side.
  UltraPerformance,
}

impl RenderScale {
  /// The ratio of the viewport's side to the drawing's.
  pub fn get_ratio(self) -> f32 {
    match self {
      Self::Native => 1.0,
      Self::UltraQuality => 1.3,
      Self::Quality => 1.5,
      Self::Balanced => 1.7,
      Self::Performance => 2.0,
      Self::UltraPerformance => 3.0,
    }
  }

  /// The DLSS quality mode drawing at about the same share: Ultra Quality draws within Quality's range, DLSS offering
  /// no mode of its own for it.
  pub fn get_dlss_quality(self) -> DlssQuality {
    match self {
      Self::Native => DlssQuality::Native,
      Self::UltraQuality | Self::Quality => DlssQuality::Quality,
      Self::Balanced => DlssQuality::Balanced,
      Self::Performance => DlssQuality::Performance,
      Self::UltraPerformance => DlssQuality::UltraPerformance,
    }
  }

  /// A viewport side's pixels as drawn, never none: the side over the ratio, truncated as FSR's and DLSS's own
  /// render sizes are, which DLSS's fixed ranges (Ultra Performance's one size) hold to.
  pub fn get_drawn(self, side: u32) -> u32 {
    ((side as f32 / self.get_ratio()) as u32).clamp(1, side.max(1))
  }
}
