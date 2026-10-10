use xrf_renderer_core::GraphTexture;

/// The guides DLSS ray reconstruction reads beside the frame and the G-buffer's albedo, at the frame's size.
#[derive(Clone, Copy)]
pub struct RayGuides {
  pub specular_albedo: GraphTexture,
  /// The world normal, the linear roughness in the fourth channel.
  pub normals: GraphTexture,
  pub hit_distance: GraphTexture,
}
