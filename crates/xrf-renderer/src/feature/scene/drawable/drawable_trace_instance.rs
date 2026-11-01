/// A drawable kind's instance in a view's own top level, after the scene's: its structure, the custom index a hit on it
/// reads (a sentinel past the scene's instance table, which the trace's shaders tell the kind by) and its mask.
#[derive(Clone, Copy, Debug)]
pub struct DrawableTraceInstance<'a> {
  pub blas: &'a wgpu::Blas,
  pub custom_index: u32,
  pub mask: u8,
}
