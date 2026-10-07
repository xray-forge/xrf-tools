use serde::{Deserialize, Serialize};

use crate::contract::render_glass_mode::RenderGlassMode;

/// How glass is drawn, and the enhanced glass's strengths.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGlassSettings {
  pub mode: RenderGlassMode,
  /// How far the enhanced glass bends what is behind it, one as designed; none draws it flat.
  pub refraction: f32,
}

impl Default for RenderGlassSettings {
  /// The engine's glass, with the enhanced glass's designed refraction for when it is drawn.
  fn default() -> Self {
    Self {
      mode: RenderGlassMode::Engine,
      refraction: 1.0,
    }
  }
}

impl RenderGlassSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderGlassMode::Enhanced
  }
}
