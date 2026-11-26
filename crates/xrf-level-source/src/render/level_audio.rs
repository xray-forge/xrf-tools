use xrf_audio::AudioServer;

use crate::sound::BurstSoundTee;

/// Where shown levels are heard: the application's audio server, and the tee that copies sound frames to a running
/// burst. A clone shares both.
///
/// The server always runs: without an output device it plays to nothing and says why in its output status, so showing,
/// recording and bursts behave the same either way.
#[derive(Clone)]
pub struct LevelAudio {
  pub server: AudioServer,
  pub bursts: BurstSoundTee,
}

impl LevelAudio {
  pub fn new(server: AudioServer) -> Self {
    Self {
      server,
      bursts: BurstSoundTee::default(),
    }
  }
}
