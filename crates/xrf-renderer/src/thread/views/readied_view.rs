use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::scene::level::level_frame::LevelFrame;
use crate::viewport::pick::drawn_pick::DrawnPick;

/// A viewport's frame readied for the frame's graph, with the pick it draws, where it draws one.
pub struct ReadiedView {
  pub viewport: RenderViewportId,
  pub frame: LevelFrame,
  pub pick: Option<DrawnPick>,
}
