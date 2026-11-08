/// A stretch of a burst's sound as it is rendered, for its video: interleaved stereo at [`crate::viewer::burst::BurstSound::RATE`],
/// its first frame the sound's frame `at` from the first frame read back, where the video's time starts.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BurstSoundChunk {
  pub at: u64,
  pub samples: Vec<f32>,
}
