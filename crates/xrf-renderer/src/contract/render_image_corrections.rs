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
