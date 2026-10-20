use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// The sky's irradiance as order-2 spherical harmonics, as `shaders/frame/sky_irradiance.wgsl` writes it: nine
/// coefficients, each in a vector's colour, of the irradiance over π in the engine's light units, already convolved by
/// the cosine; the first's `w` one once the sky was projected, nothing before.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "SkyHarmonics")]
pub struct SkyHarmonics {
  pub coefficients: [Vec4; 9],
}
