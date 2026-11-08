use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::weather::sky::sky_parameters::SkyParameters;

/// What the water's reflection traces the rays the screen left by, where the device traces rays: the sky a miss
/// samples and the level's geometry.
#[derive(Clone, Copy)]
pub(crate) struct TracedWater<'a> {
  pub sky: SkyParameters<'a>,
  pub scene: TracedScene<'a>,
}
