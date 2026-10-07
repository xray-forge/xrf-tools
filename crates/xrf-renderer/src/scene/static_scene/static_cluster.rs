use xrf_renderer_core::ShaderStruct;

/// One cluster as the cull and the vertex shader read it: up to 128 triangles of one slot.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Cluster")]
pub struct StaticCluster {
  /// Its first index, in the shared index arena.
  pub first_index: u32,
  pub triangles: u32,
  /// Its geometry's first vertex in its layout's arena, which its indices count from.
  pub vertex_start: u32,
  /// The slot it is drawn as.
  pub slot: u32,
}
