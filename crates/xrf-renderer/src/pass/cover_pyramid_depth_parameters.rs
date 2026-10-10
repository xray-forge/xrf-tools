use xrf_renderer_core::{GraphTexture, PassParameters};

/// What reducing a rain cover into its pyramid's first level reads and writes.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct CoverPyramidDepthParameters {
  #[texture(d2, depth)]
  pub source_depth: GraphTexture,
  #[binding(3)]
  #[storage_texture(d2, rg32float, write)]
  pub cover_level: GraphTexture,
}
