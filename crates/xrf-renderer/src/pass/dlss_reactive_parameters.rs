use xrf_renderer_core::{GraphTexture, PassParameters};

/// What the mask DLSS super resolution favours this frame by is worked out from: the frame as DLSS takes it, and the
/// frame as it stood before the water, the composited surfaces and the particles drew.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct DlssReactiveParameters {
  #[texture(d2, unfilterable)]
  pub frame: GraphTexture,
  #[texture(d2, unfilterable)]
  pub opaque: GraphTexture,
}
