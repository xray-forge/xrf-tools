use xrf_renderer_core::{GraphTexture, PassParameters};

/// What restoring a tile of the light shadow atlas reads: the static atlas, whose same tile holds what stands still.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct ShadowTileRestoreParameters {
  #[texture(d2, depth)]
  pub static_atlas: GraphTexture,
}
