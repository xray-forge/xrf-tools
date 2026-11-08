use xrf_renderer_core::GraphTexture;

use crate::feature::tracing::ray_guides::ray_guides::RayGuides;
use crate::feature::tracing::reflections::reflection_source::ReflectionSource;
use crate::feature::tracing::reflections::reflections_recorded::ReflectionsRecorded;
use crate::feature::weather::sky::sky_parameters::SkyParameters;

/// What a view's lighting phase hands what blends over the frame and resolves it: the indirect light, the reflections
/// traced, the sky's group, the guides DLSS ray reconstruction reads, the frame as it stood before the water and what
/// the transparent layers left of it, the depth the water's surface is resolved by, and the last frame's scene the
/// reflections keep.
#[derive(Clone, Copy)]
pub(crate) struct LightingRecorded<'a> {
  pub indirect_light: Option<GraphTexture>,
  pub reflections: Option<ReflectionsRecorded>,
  pub sky: Option<SkyParameters<'a>>,
  pub ray_guides: Option<RayGuides>,
  pub before_water: Option<GraphTexture>,
  pub opaque: Option<GraphTexture>,
  /// The G-buffer's depth with the nearest water's over it, which the resolve reprojects by; none without water.
  pub depth_with_water: Option<GraphTexture>,
  pub source: Option<ReflectionSource>,
}
