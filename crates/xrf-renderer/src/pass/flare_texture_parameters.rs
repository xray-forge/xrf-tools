use xrf_renderer_core::{GraphTexture, PassParameters};

/// One flare's texture, or the gradient's.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct FlareTextureParameters {
  #[texture(d2, float)]
  pub flare_texture: GraphTexture,
}
