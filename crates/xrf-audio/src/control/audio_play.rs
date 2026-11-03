use crate::mixer::audio_bus::AudioBus;
use crate::voice::voice_source::VoiceSource;

/// A voice to start.
#[derive(Debug)]
pub struct AudioPlay {
  pub source: VoiceSource,
  pub bus: AudioBus,
  /// Linear gain, 1 as the sound was recorded; below zero or not a number is silence.
  pub gain: f32,
  /// Playback speed, 1 at the sound's own rate: the engines' `frequency`. Not a number above zero plays at 1.
  pub rate: f32,
  /// Linear gains of the left and right outputs, both 1 for a sound heard as it was recorded.
  pub pan: [f32; 2],
}

impl AudioPlay {
  /// The sound at its own volume and speed, unplaced.
  pub fn new(source: VoiceSource, bus: AudioBus) -> Self {
    Self {
      source,
      bus,
      gain: 1.0,
      rate: 1.0,
      pan: [1.0, 1.0],
    }
  }
}
