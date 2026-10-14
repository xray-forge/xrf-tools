use glam::Vec4;

use crate::{
  AccelerationStructureBinding, GraphBuffer, GraphBufferAccess, GraphTexture, GraphTextureAccess, PassBinding,
  PassParameters, PassResources, ShaderAtomicU32, ShaderDeclarations, ShaderStruct, StorageArray, StorageArrayMut,
  StorageValue, StorageValueMut, UniformBinding,
};

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

/// Counters shaders add to atomically, and a range of a list.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct Count {
  #[storage]
  pub counters: StorageArrayMut<ShaderAtomicU32>,
  #[storage]
  pub list: StorageArray<u32>,
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

/// A value read from a buffer's head, and one read and written.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct Head {
  #[storage]
  pub settings: StorageValue<Settings>,
  #[storage]
  pub state: StorageValueMut<Settings>,
}

/// A texture and a binding array of four.
#[derive(PassParameters)]
#[parameters(group = 3)]
pub struct Environment {
  #[texture(cube, float)]
  pub sky: GraphTexture,
  #[texture(cube, float)]
  pub cubes: [GraphTexture; ENVIRONMENT_CUBES],
}

/// A structure rays are traced against and the records its hits read.
#[derive(PassParameters)]
#[parameters(group = 3)]
pub struct Traced<'a> {
  #[acceleration_structure]
  pub scene: AccelerationStructureBinding<'a>,
  #[storage]
  pub records: StorageArray<u32>,
}

/// The cubes [`Environment`] binds.
pub const ENVIRONMENT_CUBES: usize = 4;

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

/// A pass declaring a binding of its own before one that clashes as `Clashing`'s does.
#[derive(PassParameters)]
#[parameters(group = 2)]
pub struct ClashingLate {
  #[binding(5)]
  #[texture(d2, float)]
  pub ripples: GraphTexture,
  #[binding(1)]
  #[texture(d2, float)]
  pub history: GraphTexture,
}

/// Parameters written by hand that number two bindings and declare one of them.
pub struct Undeclared;

impl PassParameters for Undeclared {
  const GROUP: u32 = 3;
  const LAYOUT_KEY: &'static str = "Undeclared";
  const BINDINGS: &'static [u32] = &[0, 1];
  const ENABLES: &'static [&'static str] = &[];

  fn get_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
    Vec::new()
  }

  fn list_wgsl_bindings() -> Vec<String> {
    vec![String::from("@group(3) @binding(0) var scene: texture_2d<f32>;")]
  }

  fn declare(_: &mut ShaderDeclarations) {}

  fn list_texture_accesses(&self) -> Vec<(GraphTexture, GraphTextureAccess)> {
    Vec::new()
  }

  fn list_buffer_accesses(&self) -> Vec<(GraphBuffer, GraphBufferAccess)> {
    Vec::new()
  }

  fn list_bindings<'r>(&'r self, _: &dyn PassResources<'r>) -> Vec<PassBinding<'r>> {
    Vec::new()
  }

  fn list_dynamic_offsets(&self) -> Vec<u32> {
    Vec::new()
  }
}
