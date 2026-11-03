use std::time::Duration;

use crate::decode::sound_channels::SoundChannels;

/// What a sound's audio track is, as its header states: enough to play it before a frame of it is decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundFormat {
  pub sample_rate: u32,
  pub channels: SoundChannels,
  /// Frames each channel plays.
  pub frames_count: u64,
}

impl SoundFormat {
  pub fn get_duration(&self) -> Duration {
    self.to_duration(self.frames_count)
  }

  /// How long `frame` frames play at the sound's own rate.
  pub fn to_duration(&self, frame: u64) -> Duration {
    Duration::from_secs_f64(frame as f64 / f64::from(self.sample_rate.max(1)))
  }

  /// The frame `position` falls on, clamped to the sound.
  pub fn to_frame(&self, position: Duration) -> u64 {
    ((position.as_secs_f64() * f64::from(self.sample_rate)) as u64).min(self.frames_count)
  }
}
