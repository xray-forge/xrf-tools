use crate::frame::readback::frame_capture::{CaptureReply, FrameCapture};

/// A capture or screenshot copied out of a viewport's frame and on its way back, and where its answer goes once it is.
pub struct CaptureInFlight {
  pub capture: FrameCapture,
  pub reply: CaptureReply,
}
