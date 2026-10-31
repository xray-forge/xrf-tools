use crate::contract::render_screenshot::RenderScreenshot;
use crate::frame::frame_capture::CaptureReply;

/// A screenshot asked of a viewport, waiting for a frame: how it is drawn, and where its answer goes.
pub struct PendingScreenshot {
  pub screenshot: RenderScreenshot,
  pub reply: CaptureReply,
}
