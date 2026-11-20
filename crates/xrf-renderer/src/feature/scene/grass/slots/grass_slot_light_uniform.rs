use glam::IVec4;
use xrf_renderer_core::ShaderStruct;

/// What the slots' light view reads beside the level's grass: the slot grid's size and the world slot its first cell
/// stands for, negated, and the value it shows (`GrassSlotValue`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "GrassSlotLight")]
pub(crate) struct GrassSlotLightUniform {
  pub grid: IVec4,
  pub value: u32,
  pub _pad: [u32; 3],
}
