use xrf_renderer_core::GraphTexture;

/// The guides DLSS ray reconstruction reads beside the frame and the G-buffer's albedo, at the frame's size.
#[derive(Clone, Copy)]
pub struct RayGuides {
  pub specular_albedo: GraphTexture,
  /// The world normal, the linear roughness in the fourth channel.
  pub normals: GraphTexture,
  pub hit_distance: GraphTexture,
  /// How far each pixel's indirect light ray went, where the indirect light is traced.
  pub diffuse_hit_distance: Option<GraphTexture>,
  /// How far each pixel is reconstructed from this frame rather than the history: a puddle's rippled water.
  pub bias: GraphTexture,
  /// The frame before its transparent layers drew (the water, the composited surfaces, the particles) and after the
  /// rain did, once both are kept.
  pub transparency: Option<[GraphTexture; 2]>,
}
