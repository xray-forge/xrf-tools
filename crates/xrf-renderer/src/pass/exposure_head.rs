use xrf_renderer_core::ShaderStruct;

/// The head of the exposure's state, as the passes tonemapping with it read it: the scale, adapted frame by frame, the
/// scale the last frame's tonemap used, which a read of that frame's colour inverts by, and, under the physical light,
/// the log2 of the key the histogram metered as the scale exposes it (the windowed average of the frame's land, the
/// sky left out), which the display transforms' anchors are solved at (`DisplayMode::get_anchor_stops`).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Exposure")]
pub struct ExposureHead {
  pub adapted: f32,
  pub used: f32,
  pub key: f32,
}
