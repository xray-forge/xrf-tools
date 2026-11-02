use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What the froxel volume's passes and the shafts' composite read as their `Froxels`: the grid's froxels across, down
/// and along and the metres it reaches, none where no volume is drawn; and its scattering's strength, its phase's lean,
/// this frame's jitter along the view and one where the history holds a frame.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Froxels")]
pub struct FroxelUniform {
  pub size: Vec4,
  pub params: Vec4,
}
