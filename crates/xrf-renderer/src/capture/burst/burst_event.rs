use xrf_error::XrfError;

use crate::capture::burst::burst_marks::BurstMarks;
use crate::contract::render_capture::RenderCapture;

/// What a burst tells its caller as it goes.
#[derive(Debug)]
pub enum BurstEvent {
  /// A frame read back: its place among the frames read back, the sample of it a reference drew (none otherwise), the
  /// scene's seconds it was drawn at, the finished frame at the viewport's size and its G-buffer's marks.
  Shot {
    index: u32,
    sample: Option<u32>,
    time: f32,
    picture: RenderCapture,
    marks: BurstMarks,
  },
  /// Every frame was read back.
  Finished,
  /// It stopped, asked to or failing, and reads back nothing more.
  Stopped { error: XrfError },
}
