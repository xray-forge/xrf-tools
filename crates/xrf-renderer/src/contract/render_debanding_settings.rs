use serde::{Deserialize, Serialize};

use crate::contract::render_debanding_mode::RenderDebandingMode;
use crate::contract::render_debanding_quality::RenderDebandingQuality;

/// Whether and how the sky's colour bands are smoothed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderDebandingSettings {
  pub mode: RenderDebandingMode,
  pub quality: RenderDebandingQuality,
  /// Pixels around a sky pixel its neighbours are read from at most.
  pub radius: f32,
}

impl Default for RenderDebandingSettings {
  /// The sky as drawn, with the debanding's shipped strengths.
  fn default() -> Self {
    Self {
      mode: RenderDebandingMode::Engine,
      quality: RenderDebandingQuality::Medium,
      radius: 48.0,
    }
  }
}

impl RenderDebandingSettings {
  /// Whether the sky is debanded at all: enhanced, and its radius above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderDebandingMode::Enhanced && self.radius > 0.0
  }
}
