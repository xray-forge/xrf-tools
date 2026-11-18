use xrf_audio::AudioServer;

use crate::viewer::burst::BurstSoundTee;

/// Where shown levels are heard: the application's audio server, and the tee a burst's sound frames leave by for the
/// burst's own session. A clone shares both.
///
/// The audio plugin always starts the server: with no output device it plays to nothing and reports why in its output
/// status, so a level is shown, recorded and burst the same with or without one.
#[derive(Clone)]
pub struct LevelAudio {
  pub server: AudioServer,
  pub bursts: BurstSoundTee,
}

impl LevelAudio {
  /// Levels heard through `server`.
  pub fn new(server: AudioServer) -> Self {
    Self {
      server,
      bursts: BurstSoundTee::default(),
    }
  }
}
