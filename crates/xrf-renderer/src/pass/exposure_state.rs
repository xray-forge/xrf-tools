use xrf_renderer_core::ShaderStruct;

use crate::scene::level::feature::exposure_feature::EXPOSURE_CELLS;

/// The exposure's state as its pass adapts it: the scale the tonemap multiplies by, the scale the last frame's used,
/// then each cell's luminance.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
pub struct ExposureState {
  pub adapted: f32,
  pub used: f32,
  pub pad1: f32,
  pub pad2: f32,
  pub cells: [f32; (EXPOSURE_CELLS * EXPOSURE_CELLS) as usize],
}
