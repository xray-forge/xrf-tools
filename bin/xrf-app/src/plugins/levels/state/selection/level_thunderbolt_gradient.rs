use xrf_material::XraySurfaceDraw;

use crate::plugins::levels::state::selection::level_texture_reference::LevelTextureReference;

/// A glow a thunderbolt draws facing the view, `SThunderboltDesc::SFlare`: at the bolt's top, or at its middle.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelThunderboltGradient {
  /// Times the strike's phase.
  pub opacity: f32,
  /// Across and up, as fractions of the bolt's length.
  pub radius: [f32; 2],
  pub texture: LevelTextureReference,
  pub draw: XraySurfaceDraw,
}
