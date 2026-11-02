use xrf_renderer_core::{GraphTexture, PassParameters};

/// What reducing one of the pyramid's levels into the next reads and writes.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct PyramidLevelParameters {
  #[binding(1)]
  #[texture(d2, unfilterable)]
  pub source_level: GraphTexture,
  #[storage_texture(d2, r32float, write)]
  pub target_level: GraphTexture,
}
