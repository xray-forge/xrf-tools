use crate::scene::level::feature::traced_scene::TracedScene;
use crate::scene::level::feature::wet_targets::WetTargets;

/// What the sun's light reads of the units before it: the scene's trace, where it is built, what the wetting wrote, and
/// how many texels each way the cascades' shadows are filtered.
#[derive(Clone, Copy)]
pub struct DirectLightInputs<'a> {
  pub traced: Option<TracedScene<'a>>,
  pub wet: WetTargets,
  pub filter_reach: u32,
}
