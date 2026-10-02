use crate::scene::level::light_shadow_set::LightShadowSet;

/// What one light's shadow holds of the atlas: the faces it lights with, and those it is drawing at another size, which
/// take their place once all are drawn.
#[derive(Clone, Debug, Default)]
pub struct LightShadowEntry {
  pub shown: Option<LightShadowSet>,
  pub next: Option<LightShadowSet>,
  /// The frame the light was last in view, which room is made from the oldest of.
  pub seen: u64,
}
