use xrf_renderer_core::ShaderStruct;

/// One top-level instance of the level's acceleration structure as a traced hit reads it, by the instance's custom
/// index: the first record of its geometries, and the place it stands at, whose hemisphere lights it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceInstance")]
pub struct StaticTraceInstance {
  pub first_record: u32,
  pub place: u32,
  pub pad: [u32; 2],
}
