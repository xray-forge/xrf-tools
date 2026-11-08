use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::frame::readback::frame_capture::{CaptureReply, FrameCapture};

/// A capture or screenshot copied out of a viewport's frame, on its way back, and where its answer goes.
pub(crate) struct CaptureCopy {
  pub viewport: RenderViewportId,
  pub capture: FrameCapture,
  pub reply: CaptureReply,
}
