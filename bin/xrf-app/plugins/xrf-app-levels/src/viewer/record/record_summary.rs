use std::time::Duration;

/// A recording written: the video encoder that ran, by its name, the frames written, and the last one's end.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordSummary {
  pub encoder: String,
  pub frames: u32,
  pub duration: Duration,
}
