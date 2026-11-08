use xrf_renderer_core::{GraphBuffer, GraphTexture};

use crate::feature::tracing::ray_guides::ray_guides::RayGuides;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::feature::weather::sky::sky_views::SkyViews;

/// What the water reads beside the view's record: the sky's views, none before the sky is prepared; the frame copied
/// before it, which the enhanced water refracts and reflects; the cull's argument buffers; ray reconstruction's guides
/// where it reads them; the rings the enhanced drops leave; how much of the sun reaches each pixel; and the sky's group
/// and the scene's trace, which its rays the screen left are traced through where both are made.
pub(crate) struct WaterInputs<'a> {
  pub views: Option<&'a SkyViews>,
  pub before_water: Option<GraphTexture>,
  pub args: Vec<GraphBuffer>,
  pub guides: Option<RayGuides>,
  pub rings: GraphTexture,
  pub sun_seen: GraphTexture,
  pub sky: Option<SkyParameters<'a>>,
  pub traced: Option<TracedScene<'a>>,
}
