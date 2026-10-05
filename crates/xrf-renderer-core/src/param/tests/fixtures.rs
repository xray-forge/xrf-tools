use glam::Vec4;

use crate::{GraphTexture, PassParameters, ShaderStruct, StorageArray, StorageArrayMut, UniformBinding};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
pub struct Settings {
  pub factor: Vec4,
}

/// A uniform from the ring, an array read and an array written.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct Scale {
  #[uniform]
  pub settings: UniformBinding<Settings>,
  #[storage]
  pub input: StorageArray<u32>,
  #[storage]
  pub output: StorageArrayMut<u32>,
}

/// Every texture and sampler kind a blur needs.
#[derive(PassParameters)]
#[parameters(group = 1)]
pub struct Blur<'a> {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[texture(d2, depth)]
  pub depth: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  pub destination: GraphTexture,
  #[sampler(filtering)]
  pub linear: &'a wgpu::Sampler,
}

/// Textures alone, which need no device to build.
#[derive(PassParameters)]
#[parameters(group = 1)]
pub struct Shade {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[texture(d2, depth)]
  pub depth: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  pub destination: GraphTexture,
}

/// One of two passes drawing with one module: a texture each binds alike, then one of its own past a gap.
#[derive(PassParameters)]
#[parameters(group = 2)]
pub struct Surface {
  #[texture(d2, float)]
  pub scene: GraphTexture,
  #[binding(3)]
  #[texture(d2, float)]
  pub foam: GraphTexture,
  #[texture(d2, float)]
  pub waves: GraphTexture,
}

/// The other pass: the shared texture, and one of its own in the gap the first leaves.
#[derive(PassParameters)]
#[parameters(group = 2)]
pub struct Reflection {
  #[texture(d2, float)]
  pub scene: GraphTexture,
  #[texture(d2, depth)]
  pub depth: GraphTexture,
}

/// A pass declaring the first's gap otherwise than the second does.
#[derive(PassParameters)]
#[parameters(group = 2)]
pub struct Clashing {
  #[binding(1)]
  #[texture(d2, float)]
  pub history: GraphTexture,
}
