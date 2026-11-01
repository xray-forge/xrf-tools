use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::frame::readback::frame_capture::CaptureReply;

/// A capture a window gave no image for this frame, put back for its viewport's next presented frame.
pub struct CaptureRetry {
  pub viewport: RenderViewportId,
  pub reply: CaptureReply,
}
