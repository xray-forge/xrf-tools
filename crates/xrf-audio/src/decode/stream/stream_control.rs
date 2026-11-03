use std::sync::Arc;
use std::time::Duration;

use crate::decode::stream::stream_shared::StreamShared;

/// The control side's handle on a stream a voice plays.
#[derive(Clone, Debug)]
pub struct StreamControl {
  shared: Arc<StreamShared>,
}

impl StreamControl {
  pub(crate) fn new(shared: Arc<StreamShared>) -> Self {
    Self { shared }
  }

  /// Whether the stream starts over at its end; takes effect from the next time the decoder reaches the end.
  pub fn set_looped(&self, is_looped: bool) {
    self.shared.set_looped(is_looped);
  }

  pub fn is_looped(&self) -> bool {
    self.shared.is_looped()
  }

  /// Why the decoder thread stopped before the sound's end, if it did.
  pub fn get_failure(&self) -> Option<String> {
    self.shared.get_failure()
  }

  pub fn get_sample_rate(&self) -> u32 {
    self.shared.sample_rate
  }

  pub fn get_frames_count(&self) -> u64 {
    self.shared.frames_count
  }

  pub fn get_duration(&self) -> Duration {
    Duration::from_secs_f64(self.shared.frames_count as f64 / f64::from(self.shared.sample_rate))
  }
}
