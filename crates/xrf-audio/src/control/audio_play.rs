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
  /// The frame of the mixer's clock ([`crate::AudioMixerMeter::get_frames`]) the source's first frame plays at: one
  /// still ahead when the mixer takes the voice up is waited for in silence, to the frame, and one passed already is
  /// played from as far on as it would be. None plays it from the block the mixer takes it up in.
  pub at: Option<u64>,
}

impl AudioPlay {
  /// The sound at its own volume and speed, unplaced, from the next block.
  pub fn new(source: VoiceSource, bus: AudioBus) -> Self {
    Self {
      source,
      bus,
      gain: 1.0,
      rate: 1.0,
      pan: [1.0, 1.0],
      at: None,
    }
  }
}
