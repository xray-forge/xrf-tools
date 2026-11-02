use xrf_renderer_core::ShaderStruct;

/// What every light's shafts share: the noise they are broken by (`fx\fx_noise`), by its slot, and how far it has
/// scrolled, the engine's `timers.z * 0.1`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "LightVolumes")]
pub struct LightVolumesUniform {
  pub noise: u32,
  pub scroll: f32,
}
