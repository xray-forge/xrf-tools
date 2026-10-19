use serde::{Deserialize, Serialize};

use crate::contract::render_debanding_mode::RenderDebandingMode;

/// Whether and how the sky's colour bands are smoothed.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderDebandingSettings {
  pub mode: RenderDebandingMode,
  /// Times a sky pixel is averaged with its neighbours, each time further out.
  pub passes: u32,
  /// Pixels around a sky pixel its neighbours are read from at most.
  pub radius: f32,
}

impl Default for RenderDebandingSettings {
  /// The sky as drawn, with the debanding's shipped strengths.
  fn default() -> Self {
    Self {
      mode: RenderDebandingMode::Engine,
      passes: 2,
      radius: 48.0,
    }
  }
}

impl RenderDebandingSettings {
  /// The least and most times a pixel is averaged.
  pub const PASSES_RANGE: (u32, u32) = (1, 4);

  pub fn get_passes(&self) -> u32 {
    self.passes.clamp(Self::PASSES_RANGE.0, Self::PASSES_RANGE.1)
  }

  /// Whether the sky is debanded at all: enhanced, and its radius above none.
  pub fn is_drawn(&self) -> bool {
    self.mode == RenderDebandingMode::Enhanced && self.radius > 0.0
  }
}
