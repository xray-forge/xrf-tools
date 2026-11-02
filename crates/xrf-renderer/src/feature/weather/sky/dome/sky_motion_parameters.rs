use xrf_renderer_core::{GraphTexture, PassParameters};

/// What the sky's motion reads: the frame's depth, which says where nothing was drawn.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SkyMotionParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
}
