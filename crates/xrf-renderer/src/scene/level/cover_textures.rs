use xrf_renderer_core::GraphTexture;

/// The rain cover's depth and pyramid as the graph knows them.
#[derive(Clone, Copy)]
pub struct CoverTextures {
  pub depth: GraphTexture,
  pub pyramid: GraphTexture,
}
