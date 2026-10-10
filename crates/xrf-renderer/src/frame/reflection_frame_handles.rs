use xrf_renderer_core::GraphTexture;

/// A reflection history's textures as one frame's graph holds them, each the side read then the side written.
#[derive(Clone, Copy)]
pub struct ReflectionFrameHandles {
  pub radiance: [GraphTexture; 2],
  pub surface: [GraphTexture; 2],
  pub held: [GraphTexture; 2],
  pub full: [GraphTexture; 2],
}
