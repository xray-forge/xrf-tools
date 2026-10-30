use glam::UVec4;
use xrf_renderer_core::ShaderStruct;

/// Where compressing a level of a bake's maps writes, as its `ImpostorEncode` reads it: the level's texels a side, its
/// blocks a side, and the blocks a row of the block buffer takes (padded to a copy's row alignment); then where each
/// map's blocks of the level start in the buffer, in blocks.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "ImpostorEncode")]
pub struct ImpostorEncodeUniform {
  pub size: UVec4,
  pub starts: UVec4,
}
