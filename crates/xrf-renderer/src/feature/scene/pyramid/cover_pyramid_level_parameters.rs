use xrf_renderer_core::{GraphTexture, PassParameters};

/// What reducing one of a rain cover pyramid's levels into the next reads and writes.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct CoverPyramidLevelParameters {
  #[binding(3)]
  #[storage_texture(d2, rg32float, write)]
  pub cover_level: GraphTexture,
  #[texture(d2, unfilterable)]
  pub cover_source: GraphTexture,
}
