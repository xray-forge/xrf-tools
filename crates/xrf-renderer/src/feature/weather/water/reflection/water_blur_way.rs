use xrf_renderer_core::GraphTexture;

use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::weather::water::reflection::water_blur_uniform::WaterBlurUniform;

/// One way of the reflection's blur: its pass's name, the texture it blurs, the size it blurs into, which way, and the
/// rectangles of its target it blurs within.
pub struct WaterBlurWay {
  pub name: &'static str,
  pub source: GraphTexture,
  pub size: (u32, u32),
  pub way: WaterBlurUniform,
  pub rects: Vec<RenderRect>,
}
