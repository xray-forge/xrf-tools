/// A rendered stretch of a burst's sound for its video: interleaved stereo at
/// [`crate::viewer::burst::BurstSound::RATE`], starting `at` sound frames after the first frame read back, where the
/// video's time starts.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BurstSoundChunk {
  pub at: u64,
  pub samples: Vec<f32>,
}
