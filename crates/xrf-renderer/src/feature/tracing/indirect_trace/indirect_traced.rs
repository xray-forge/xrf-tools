use xrf_renderer_core::GraphTexture;

/// The indirect light traced in a frame: the light, and how far each pixel's ray went, which ray reconstruction reads.
#[derive(Clone, Copy)]
pub struct IndirectTraced {
  pub light: GraphTexture,
  pub hits: GraphTexture,
}
