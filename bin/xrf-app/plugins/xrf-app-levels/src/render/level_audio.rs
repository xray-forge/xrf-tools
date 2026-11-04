use xrf_audio::AudioServer;

use crate::viewer::burst::BurstSoundTee;

/// Where shown levels are heard: the application's audio server, none where it did not start, and the tee a burst's
/// sound frames leave by for the burst's own session. A clone shares both.
#[derive(Clone, Default)]
pub struct LevelAudio {
  pub server: Option<AudioServer>,
  pub bursts: BurstSoundTee,
}

impl LevelAudio {
  /// Levels heard through `server`.
  pub fn new(server: Option<AudioServer>) -> Self {
    Self {
      server,
      bursts: BurstSoundTee::default(),
    }
  }
}
