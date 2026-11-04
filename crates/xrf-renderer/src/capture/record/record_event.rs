use xrf_error::XrfError;

use crate::capture::record::record_frame::RecordFrame;

/// What a recording tells its recorder as it goes.
#[derive(Debug)]
pub enum RecordEvent {
  /// A frame kept and back from the GPU, for the recorder to read.
  Frame(RecordFrame),
  /// It stopped, its viewport or the GPU gone, and keeps nothing more.
  Stopped { error: XrfError },
}
