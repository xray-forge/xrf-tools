use xrf_error::XrfError;

use crate::capture::capture_shot::CaptureShot;
use crate::contract::render_capture::RenderCapture;

/// What a capture tells its caller as it goes.
#[derive(Debug)]
pub enum CaptureEvent {
  /// How many of its shots are drawn, of how many: the metering first, counted as none.
  Progress { drawn: u32, total: u32 },
  /// A shot read back at its drawn size, a map's tile with its coverage in alpha, and which it was.
  Shot { shot: CaptureShot, capture: RenderCapture },
  /// Every shot was read back.
  Finished,
  /// It stopped, asked to or failing, and reads back nothing more.
  Stopped { error: XrfError },
}
