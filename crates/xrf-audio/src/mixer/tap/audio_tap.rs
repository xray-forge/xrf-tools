use std::sync::Arc;
use std::time::Duration;

use rtrb::{Consumer, RingBuffer};

use crate::mixer::tap::audio_tap_block::AudioTapBlock;
use crate::mixer::tap::audio_tap_end::AudioTapEnd;
use crate::mixer::tap::mixer_tap::MixerTap;
use crate::mixer::tap::tap_shared::TapShared;

/// The mix taken after the reverb and before the master, so the app's master volume and background mute stay out of
/// it: blocks of the mixer's frames read from any one thread, through a ring as long as asked; a block the reader had
/// no room for is dropped and counted, its frames missing between the blocks either side. Dropped, the tap leaves the
/// mixer at its next block.
pub struct AudioTap {
  blocks: Consumer<AudioTapBlock>,
  shared: Arc<TapShared>,
}

impl AudioTap {
  /// A tap holding `length` of the mix at `rate`, and the end the mixer pushes into.
  pub(crate) fn new(length: Duration, rate: u32) -> (Self, MixerTap) {
    let frames: usize = (length.as_secs_f64() * f64::from(rate.max(1))).round() as usize;
    let capacity: usize = frames.div_ceil(AudioTapBlock::FRAMES).max(1);
    let (producer, blocks) = RingBuffer::new(capacity);
    let shared: Arc<TapShared> = Arc::default();

    shared.set_rate(rate);

    (
      Self {
        blocks,
        shared: Arc::clone(&shared),
      },
      MixerTap::new(producer, shared),
    )
  }

  /// The next block of the mix, none while the mixer has not rendered it yet.
  ///
  /// # Errors
  ///
  /// Returns why the stream ended once every block before the end is read.
  pub fn read(&mut self) -> Result<Option<AudioTapBlock>, AudioTapEnd> {
    match self.blocks.pop() {
      Ok(block) => Ok(Some(block)),
      Err(_) => self.shared.get_end().map_or(Ok(None), Err),
    }
  }

  /// Frames a second of the tap's blocks.
  pub fn get_rate(&self) -> u32 {
    self.shared.get_rate()
  }

  /// Blocks dropped for want of room.
  pub fn get_dropped(&self) -> u64 {
    self.shared.get_dropped()
  }
}
