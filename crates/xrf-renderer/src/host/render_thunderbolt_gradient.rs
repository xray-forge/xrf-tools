use xrf_material::XraySurfaceDraw;

/// A glow a bolt draws facing the view, `SThunderboltDesc::SFlare`: at its top, or at its middle.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderThunderboltGradient {
  /// Times the strike's phase.
  pub opacity: f32,
  /// Across and up, as fractions of the bolt's length.
  pub radius: [f32; 2],
  pub texture: String,
  /// How its shader composites it.
  pub draw: XraySurfaceDraw,
}
