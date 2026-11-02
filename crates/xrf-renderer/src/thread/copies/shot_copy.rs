use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::frame::readback::frame_capture::FrameCapture;

/// A capture's shot copied out of a viewport's frame, on its way back.
pub struct ShotCopy {
  pub viewport: RenderViewportId,
  pub capture: FrameCapture,
}
