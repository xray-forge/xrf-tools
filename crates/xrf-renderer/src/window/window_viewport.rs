use crate::contract::viewport::render_rect::RenderRect;
use crate::contract::viewport::render_viewport_id::RenderViewportId;

/// A viewport a window draws this frame: its whole rectangle, which its camera and targets are sized by, and the part
/// of it the window shows, which its picture is cropped to; both empty for a capture laid out nowhere, which draws into
/// a texture of its own.
#[derive(Clone, Copy)]
pub struct WindowViewport {
  pub id: RenderViewportId,
  pub rect: RenderRect,
  pub shown: RenderRect,
}
