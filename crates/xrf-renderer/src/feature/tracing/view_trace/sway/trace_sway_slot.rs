use glam::Mat4;
use xrf_renderer_core::ShaderStruct;

/// One swaying tree instance the trace's sway pass writes, as `shaders/static/trace_sway.wgsl` reads it: what turns the
/// renderer's space into its tree's own, which its instance stands it out of again, its geometry's vertices in the tree
/// arena, its place, its surface, and where its swayed vertices start in the pass's output, which the slots run through
/// in order.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceSwaySlot")]
pub(crate) struct TraceSwaySlot {
  pub to_local: Mat4,
  pub vertex_start: u32,
  pub vertex_count: u32,
  pub place: u32,
  pub surface: u32,
  /// Its first vertex in the output.
  pub first_out: u32,
  pub _pad: [u32; 3],
}
