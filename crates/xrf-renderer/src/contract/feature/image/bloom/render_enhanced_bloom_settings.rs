use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::image::bloom::render_bloom_mode::RenderBloomMode;

/// Which bloom the frame draws, and the enhanced bloom's strengths; the engine bloom's own come with the look.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderEnhancedBloomSettings {
  // The enhanced bloom's own menu bounds.
  pub mode: RenderBloomMode,
  /// The power the finished frame is raised to before it blooms: the higher, the brighter a part must be to glow.
  #[setting(min = 1, max = 10)]
  pub threshold: f32,
  /// How bright the bloom is tonemapped.
  #[setting(min = 1, max = 100)]
  pub exposure: f32,
  /// How far apart each halving size's reads are spread, in its texels: the bloom's width.
  #[setting(min = 1, max = 5)]
  pub blur: f32,
  /// How saturated the bloom's colour is: one as it is, none grey.
  #[setting(min = 0, max = 10)]
  pub vibrance: f32,
  /// How much the sky blooms, against the rest of the frame.
  #[setting(min = 0, max = 10)]
  pub sky: f32,
}

impl Default for RenderEnhancedBloomSettings {
  /// The engine's bloom, with the enhanced bloom's shipped strengths but a threshold of five.
  fn default() -> Self {
    Self {
      mode: RenderBloomMode::Engine,
      threshold: 5.0,
      exposure: 3.0,
      blur: 3.0,
      vibrance: 1.0,
      sky: 0.6,
    }
  }
}

impl RenderEnhancedBloomSettings {
  pub fn is_enhanced(&self) -> bool {
    self.mode == RenderBloomMode::Enhanced
  }
}
