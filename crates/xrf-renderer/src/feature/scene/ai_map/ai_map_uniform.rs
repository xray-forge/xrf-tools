use glam::{UVec4, Vec4};
use xrf_renderer_core::ShaderStruct;

/// What the AI map's view reads beside the map's grid and nodes: the grid's lowest corner along x and z, a cell's side
/// and the height a packed height of 0 stands at; the cells along z and along x, the field shown (`AiMapField`) and the
/// nodes held; and how far above the floor a packed height of 65535 stands.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "AiMap")]
pub(crate) struct AiMapUniform {
  pub origin: Vec4,
  pub cells: UVec4,
  pub height: f32,
  pub _pad: [f32; 3],
}
