use serde::{Deserialize, Serialize};

use crate::contract::render_bloom_mode::RenderBloomMode;

/// Which bloom the frame draws, and the enhanced bloom's strengths; the engine bloom's own come with the look.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderEnhancedBloomSettings {
  pub mode: RenderBloomMode,
  /// The power the finished frame is raised to before it blooms: the higher, the brighter a part must be to glow.
  pub threshold: f32,
  /// How bright the bloom is tonemapped.
  pub exposure: f32,
  /// How far apart each halving size's reads are spread, in its texels: the bloom's width.
  pub blur: f32,
  /// How saturated the bloom's colour is: one as it is, none grey.
  pub vibrance: f32,
  /// How much the sky blooms, against the rest of the frame.
  pub sky: f32,
}

impl Default for RenderEnhancedBloomSettings {
  /// The engine's bloom, with the enhanced bloom's shipped strengths.
  fn default() -> Self {
    Self {
      mode: RenderBloomMode::Engine,
      threshold: 3.5,
      exposure: 3.0,
      blur: 3.0,
      vibrance: 1.5,
      sky: 0.6,
    }
  }
}

impl RenderEnhancedBloomSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderBloomMode::Enhanced
  }
}
