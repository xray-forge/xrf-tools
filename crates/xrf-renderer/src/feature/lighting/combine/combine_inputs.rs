use xrf_renderer_core::GraphTexture;

use crate::feature::weather::wet::wet_targets::WetTargets;

/// What the units before combine answered for it to read: the indirect light and the reflections, none where they
/// are not drawn, what the wetting wrote, and how much of the sun each pixel sees.
#[derive(Clone, Copy)]
pub struct CombineInputs {
  pub indirect_light: Option<GraphTexture>,
  pub reflections: Option<GraphTexture>,
  pub wet: WetTargets,
  pub sun_seen: GraphTexture,
}
