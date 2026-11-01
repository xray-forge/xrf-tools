use xrf_renderer_core::GraphTexture;

/// What the bitmask search wrote and what it is filtered into: its visibility and the occlusion target combine reads,
/// and, while it gathers light, the light and the indirect light filtered from it.
#[derive(Clone, Copy)]
pub struct BitmaskTargets {
  pub searched: GraphTexture,
  pub filtered_occlusion: GraphTexture,
  pub lit_searched: Option<GraphTexture>,
  pub lit_filtered: Option<GraphTexture>,
}
