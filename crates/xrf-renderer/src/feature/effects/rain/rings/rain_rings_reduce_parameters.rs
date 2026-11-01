use xrf_renderer_core::{GraphTexture, PassParameters};

/// What reducing the rain's rings reads and writes: their first level, and the four after it.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct RainRingsReduceParameters {
  #[texture(d2, float)]
  pub rings_first: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  pub rings_second: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  pub rings_third: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  pub rings_fourth: GraphTexture,
  #[storage_texture(d2, rgba16float, write)]
  pub rings_fifth: GraphTexture,
}
