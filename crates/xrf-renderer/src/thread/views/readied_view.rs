use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::scene::level::view::record::readied_frame::ReadiedFrame;
use crate::viewport::pick::drawn_pick::DrawnPick;

/// A viewport's frame readied for the frame's graph, with the pick it draws, where it draws one.
pub(crate) struct ReadiedView {
  pub viewport: RenderViewportId,
  pub frame: ReadiedFrame,
  pub pick: Option<DrawnPick>,
}
