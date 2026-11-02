use xrf_renderer_core::{GraphTexture, PassParameters};

/// What reducing the depth into the pyramid's first level reads and writes.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct PyramidDepthParameters {
  #[texture(d2, depth)]
  pub source_depth: GraphTexture,
  #[binding(2)]
  #[storage_texture(d2, r32float, write)]
  pub target_level: GraphTexture,
}
