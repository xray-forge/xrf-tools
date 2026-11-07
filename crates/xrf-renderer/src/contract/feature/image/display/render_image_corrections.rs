use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

/// Anomaly's `img_corrections`, which `combine_2` applies to the finished frame: `r__exposure`, `r__gamma`,
/// `r__saturation` and `r__color_grading`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderImageCorrections {
  /// What the frame is multiplied by.
  pub exposure: f32,
  /// The power the frame is raised to, inverted.
  pub gamma: f32,
  /// How far from grey towards the frame's own colour: one leaves it.
  pub saturation: f32,
  /// The colour the mid tones are graded towards; black grades nothing.
  pub grading: [f32; 3],
}

impl RenderImageCorrections {
  /// The same, each held within the range Anomaly's console takes it in (`xrRender_console.cpp`), and a value that is
  /// no finite number taken as nothing corrected: no gamma of nought for the present to divide by.
  pub fn clamped(&self) -> Self {
    let neutral: Self = Self::default();
    let hold = |value: f32, otherwise: f32, range: RangeInclusive<f32>| {
      if value.is_finite() {
        value.clamp(*range.start(), *range.end())
      } else {
        otherwise
      }
    };

    Self {
      exposure: hold(self.exposure, neutral.exposure, 0.5..=4.0),
      gamma: hold(self.gamma, neutral.gamma, 0.5..=2.2),
      saturation: hold(self.saturation, neutral.saturation, 0.0..=2.0),
      grading: std::array::from_fn(|channel| hold(self.grading[channel], neutral.grading[channel], 0.0..=1.0)),
    }
  }
}

impl Default for RenderImageCorrections {
  /// The engine's own: nothing corrected.
  fn default() -> Self {
    Self {
      exposure: 1.0,
      gamma: 1.0,
      saturation: 1.0,
      grading: [0.0; 3],
    }
  }
}
