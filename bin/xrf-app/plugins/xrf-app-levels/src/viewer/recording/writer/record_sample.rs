use std::time::Duration;

/// A sample handed to a recording, timed on its own clock from zero.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RecordSample {
  /// A BGRA frame at the recording's size, rows top down, shown from `time` for `duration`.
  Frame {
    bgra: Vec<u8>,
    time: Duration,
    duration: Duration,
  },
  /// Interleaved sound in -1..=1, starting at the recording's sound frame `at`.
  Sound { samples: Vec<f32>, at: u64 },
}
