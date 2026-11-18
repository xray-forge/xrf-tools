use glam::{UVec4, Vec4};
use xrf_renderer_core::ShaderStruct;

/// What the AI map's view reads beside the map's grid and nodes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "AiMap")]
pub(crate) struct AiMapUniform {
  /// The grid's lowest corner along x and z, a cell's side, and the height a packed height of 0 stands at.
  pub origin: Vec4,
  /// The cells along z and along x, the field shown (`AiMapField`), and the nodes held.
  pub cells: UVec4,
  /// How far above the floor a packed height of 65535 stands.
  pub height: f32,
  pub _pad: [f32; 3],
}
