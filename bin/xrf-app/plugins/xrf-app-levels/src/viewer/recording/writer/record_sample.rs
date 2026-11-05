use std::time::Duration;

/// What a recording is handed, on its own clock from zero.
#[derive(Clone, Debug, PartialEq)]
pub enum RecordSample {
  /// A frame's pixels at the recording's size, rows top down, four bytes a pixel in blue, green, red, alpha order;
  /// shown from `time` for `duration`.
  Frame {
    bgra: Vec<u8>,
    time: Duration,
    duration: Duration,
  },
  /// Interleaved sound in -1 to 1, its first frame the recording's sound frame `at`.
  Sound { samples: Vec<f32>, at: u64 },
}
