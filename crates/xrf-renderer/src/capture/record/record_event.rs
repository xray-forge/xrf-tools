use xrf_error::XrfError;

use crate::capture::record::record_frame::RecordFrame;

/// A recording's message to its recorder.
#[derive(Debug)]
pub enum RecordEvent {
  /// A frame back from the GPU, ready to read.
  Frame(RecordFrame),
  /// The recording stopped, for instance because its viewport or the GPU went; no more frames follow.
  Stopped { error: XrfError },
}
