use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};

use crate::voice::playback::audio_voice_playback::AudioVoicePlayback;
use crate::voice::playback::audio_voice_state::AudioVoiceState;

/// What the audio thread last reported of the voice in one slot, read without a lock by the control side.
#[derive(Debug, Default)]
pub(crate) struct VoiceStatus {
  generation: AtomicU32,
  state: AtomicU8,
  frame: AtomicU64,
}

impl VoiceStatus {
  pub(crate) fn write(&self, generation: u32, playback: AudioVoicePlayback) {
    self.state.store(playback.state.to_code(), Ordering::Relaxed);
    self.frame.store(playback.frame, Ordering::Relaxed);
    self.generation.store(generation, Ordering::Release);
  }

  /// Where the voice `generation` of the slot is, or `None` when the audio thread has not reported it yet.
  pub(crate) fn read(&self, generation: u32) -> Option<AudioVoicePlayback> {
    (self.generation.load(Ordering::Acquire) == generation).then(|| AudioVoicePlayback {
      state: AudioVoiceState::from_code(self.state.load(Ordering::Relaxed)),
      frame: self.frame.load(Ordering::Relaxed),
    })
  }
}
