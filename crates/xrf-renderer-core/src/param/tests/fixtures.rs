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
pub struct Scale<'a> {
  #[uniform]
  pub settings: UniformBinding<'a, Settings>,
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
