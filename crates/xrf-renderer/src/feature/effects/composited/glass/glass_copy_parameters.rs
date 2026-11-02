use xrf_renderer_core::{GraphTexture, PassParameters};

/// What the glass copy reads and writes: the frame as it stands before the glass, and the copy the glass refracts.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct GlassCopyParameters {
  #[texture(d2, float)]
  pub scene: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  pub copied: GraphTexture,
}
