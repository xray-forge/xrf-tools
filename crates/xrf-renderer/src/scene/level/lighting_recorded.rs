use xrf_renderer_core::GraphTexture;

use crate::pass::sky_parameters::SkyParameters;
use crate::scene::level::feature::ray_guides::RayGuides;

/// What a view's lighting phase hands what blends over the frame and resolves it: the indirect light, the reflections
/// traced and how their rays ended, the sky's group, the guides DLSS ray reconstruction reads, the frame as it stood
/// before the water and what the transparent layers left of it, and the last frame's scene the reflections keep.
#[derive(Clone, Copy)]
pub struct LightingRecorded<'a> {
  pub indirect_light: Option<GraphTexture>,
  pub reflections: Option<GraphTexture>,
  pub reflection_ends: Option<GraphTexture>,
  pub sky: Option<SkyParameters<'a>>,
  pub ray_guides: Option<RayGuides>,
  pub before_water: Option<GraphTexture>,
  pub opaque: Option<GraphTexture>,
  /// The last frame's scene and depth the reflections keep, written again once the particles draw.
  pub source: Option<(GraphTexture, GraphTexture)>,
}
