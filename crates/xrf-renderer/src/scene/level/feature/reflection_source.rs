use xrf_renderer_core::GraphTexture;

/// The last frame's scene and depth the reflections keep, as the frame's graph holds them: read by this frame's hits,
/// written again once the particles draw.
#[derive(Clone, Copy)]
pub struct ReflectionSource {
  pub scene: GraphTexture,
  pub depth: GraphTexture,
}
