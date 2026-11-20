use std::time::Duration;

/// A written recording: the video encoder's name, the frames written, and the last one's end.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RecordSummary {
  pub encoder: String,
  pub frames: u32,
  pub duration: Duration,
}
