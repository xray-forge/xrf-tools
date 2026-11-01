use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What shifting an overhead map reads as its `OverheadShift`: the texels a texel of the map now reads its old self
/// from, across and down, and the texels the map is across.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "OverheadShift")]
pub struct OverheadShiftUniform {
  pub offset: Vec4,
}

impl OverheadShiftUniform {
  pub fn new((across, down): (i32, i32), resolution: u32) -> Self {
    Self {
      offset: Vec4::new(across as f32, down as f32, resolution as f32, 0.0),
    }
  }
}
