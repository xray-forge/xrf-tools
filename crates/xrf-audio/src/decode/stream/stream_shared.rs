use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::decode::sound_channels::SoundChannels;
use crate::decode::sound_decoder::SoundDecoder;

/// What a stream's decoder thread, the voice reading it and its control share.
#[derive(Debug)]
pub(crate) struct StreamShared {
  pub(crate) sample_rate: u32,
  pub(crate) channels: SoundChannels,
  pub(crate) frames_count: u64,
  is_looped: AtomicBool,
  /// Set once the thread has pushed its last sample: the sound ended and does not loop, or it failed.
  is_ended: AtomicBool,
  /// Why the thread stopped early. Written once by the thread and read by the control side, never by the audio thread.
  failure: Mutex<Option<String>>,
}

impl StreamShared {
  pub(crate) fn new(decoder: &SoundDecoder, is_looped: bool) -> Self {
    Self {
      sample_rate: decoder.get_sample_rate(),
      channels: decoder.get_channels(),
      frames_count: decoder.get_frames_count(),
      is_looped: AtomicBool::new(is_looped),
      is_ended: AtomicBool::new(false),
      failure: Mutex::new(None),
    }
  }

  pub(crate) fn is_looped(&self) -> bool {
    self.is_looped.load(Ordering::Relaxed)
  }

  pub(crate) fn set_looped(&self, is_looped: bool) {
    self.is_looped.store(is_looped, Ordering::Relaxed);
  }

  pub(crate) fn is_ended(&self) -> bool {
    self.is_ended.load(Ordering::Acquire)
  }

  pub(crate) fn end(&self) {
    self.is_ended.store(true, Ordering::Release);
  }

  pub(crate) fn fail(&self, failure: String) {
    *self.failure.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(failure);
    self.end();
  }

  pub(crate) fn get_failure(&self) -> Option<String> {
    self
      .failure
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone()
  }
}
