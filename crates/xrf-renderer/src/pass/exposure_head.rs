use xrf_renderer_core::ShaderStruct;

/// The head of the exposure's state, as the passes tonemapping with it read it: the scale, adapted frame by frame, the
/// scale the last frame's tonemap used, which a read of that frame's colour inverts by, and, under the physical light,
/// the log2 of the key the histogram metered as the scale exposes it (the windowed average of the frame's land, the
/// sky left out), which the display transforms' anchors are solved at (`DisplayMode::get_anchor_stops`), and the log2
/// of the sky's level as the game paints it, the pictures' metered average, which they are shown by (`PictureShares`),
/// not a number before any sky was seen; then the log2 of the frame's bright end as the scale exposes it
/// (`ExposureHistogramUniform::BRIGHT_SHARE` of it under) and of the sky's bright end as it is painted (its mean and
/// 2.33 of its spread, as far up a normal spread), which the transforms never show brighter than the engine's curve.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Exposure")]
pub struct ExposureHead {
  pub adapted: f32,
  pub used: f32,
  pub key: f32,
  pub sky: f32,
  pub bright: f32,
  pub sky_bright: f32,
}
