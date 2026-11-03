use std::sync::Arc;

use rtrb::Consumer;

use crate::decode::sound_channels::SoundChannels;
use crate::decode::stream::stream_control::StreamControl;
use crate::decode::stream::stream_shared::StreamShared;
use crate::voice::cursor::stereo_frame::StereoFrame;

/// The audio thread's end of a stream: frames its decoder thread has ready, in the sound's own channels, taken without
/// a lock.
///
/// Dropping it tells the decoder thread to stop; it must be dropped off the audio thread, as the mixer hands it back.
#[derive(Debug)]
pub struct StreamReader {
  samples: Consumer<f32>,
  shared: Arc<StreamShared>,
  start_frame: u64,
}

impl StreamReader {
  pub(crate) fn new(samples: Consumer<f32>, shared: Arc<StreamShared>, start_frame: u64) -> Self {
    Self {
      samples,
      shared,
      start_frame,
    }
  }

  /// A handle the control side keeps to loop the stream or read why it failed.
  pub fn get_control(&self) -> StreamControl {
    StreamControl::new(self.shared.clone())
  }

  pub(crate) fn get_sample_rate(&self) -> u32 {
    self.shared.sample_rate
  }

  pub(crate) fn get_frames_count(&self) -> u64 {
    self.shared.frames_count
  }

  /// The source frame the stream started at.
  pub(crate) fn get_start_frame(&self) -> u64 {
    self.start_frame
  }

  pub(crate) fn get_channels(&self) -> SoundChannels {
    self.shared.channels
  }

  /// The next frame, or `None` when the decoder thread has not produced it yet or the stream ended.
  pub(crate) fn pop_frame(&mut self) -> Option<StereoFrame> {
    if self.samples.slots() < self.shared.channels.get_count() {
      return None;
    }

    let left: f32 = self.samples.pop().ok()?;
    let right: f32 = match self.shared.channels {
      SoundChannels::Mono => left,
      SoundChannels::Stereo => self.samples.pop().ok()?,
    };

    Some(StereoFrame { left, right })
  }

  /// Whether no frame will follow: the decoder thread ended and every frame it pushed was taken.
  pub(crate) fn is_drained(&self) -> bool {
    self.shared.is_ended() && self.samples.slots() < self.shared.channels.get_count()
  }
}
