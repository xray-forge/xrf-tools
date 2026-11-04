use crate::mixer::audio_mixer::AudioMixer;

/// One block of the mix as a tap takes it, after the reverb and before the master: its first frame on the mixer's
/// clock, the clock voices start on, and its stereo pairs interleaved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioTapBlock {
  pub frame: u64,
  pub samples: [f32; AudioTapBlock::SAMPLES],
}

impl AudioTapBlock {
  /// Frames a block holds, the mixer's.
  pub const FRAMES: usize = AudioMixer::BLOCK_FRAMES;

  /// Samples a block holds, two a frame.
  pub const SAMPLES: usize = Self::FRAMES * 2;
}
