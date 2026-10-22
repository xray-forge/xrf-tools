use xrf_renderer_core::ShaderStruct;

use crate::scene::level::feature::exposure_feature::ExposureFeature;

/// The exposure's state as its pass adapts it: the scale the tonemap multiplies by, the scale the last frame's used,
/// what the physical light's histogram metered (`ExposureHead`), then each cell's luminance.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
pub struct ExposureState {
  pub adapted: f32,
  pub used: f32,
  pub key: f32,
  pub sky: f32,
  pub bright: f32,
  pub sky_bright: f32,
  pub cells: [f32; (ExposureFeature::CELLS * ExposureFeature::CELLS) as usize],
}
