use serde::{Deserialize, Serialize};

/// How much smaller than the viewport the scene is drawn and then upscaled: FSR's quality modes, by the ratio of the
/// viewport's side to the drawing's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderScale {
  #[default]
  Native,
  /// 1.5: two thirds of each side.
  Quality,
  /// 1.7.
  Balanced,
  /// 2: half of each side.
  Performance,
}

impl RenderScale {
  /// The ratio of the viewport's side to the drawing's.
  pub fn get_ratio(self) -> f32 {
    match self {
      Self::Native => 1.0,
      Self::Quality => 1.5,
      Self::Balanced => 1.7,
      Self::Performance => 2.0,
    }
  }

  /// A viewport side's pixels as drawn: never none.
  pub fn get_drawn(self, side: u32) -> u32 {
    ((side as f32 / self.get_ratio()).ceil() as u32).clamp(1, side.max(1))
  }
}
