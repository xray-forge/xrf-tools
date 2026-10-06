use crate::contract::render_level_hit::RenderLevelHit;

/// What a pick met, and the viewport's frame it was drawn in, counted from its first: the answer comes a frame or
/// more after it.
#[derive(Clone, Debug)]
pub struct RenderPick {
  pub frame: u64,
  pub hit: Option<RenderLevelHit>,
}
