use xrf_renderer_core::GraphTexture;

use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::weather::water::reflection::water_blur_uniform::WaterBlurUniform;

/// One way of the reflection's blur.
pub(crate) struct WaterBlurWay {
  pub name: &'static str,
  pub source: GraphTexture,
  /// The size it blurs into.
  pub size: (u32, u32),
  pub way: WaterBlurUniform,
  /// The rectangles of its target it blurs within.
  pub rects: Vec<RenderRect>,
}
