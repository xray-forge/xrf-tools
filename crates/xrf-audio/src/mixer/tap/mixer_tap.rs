use std::sync::Arc;

use rtrb::Producer;

use crate::mixer::tap::audio_tap_block::AudioTapBlock;
use crate::mixer::tap::tap_shared::TapShared;

/// A tap's audio-thread end: each block of the mix goes into its ring while there is room, else it is counted as
/// dropped; nothing here allocates or frees. Made on the control side with its reader ([`crate::AudioTap`]) and
/// handed back there to be freed once the reader has let it go or another tap took its place.
#[derive(Debug)]
pub(crate) struct MixerTap {
  blocks: Producer<AudioTapBlock>,
  shared: Arc<TapShared>,
}

impl MixerTap {
  pub(crate) fn new(blocks: Producer<AudioTapBlock>, shared: Arc<TapShared>) -> Self {
    Self { blocks, shared }
  }

  /// Takes the mix from the next block on, at `rate` frames a second.
  pub(crate) fn start(&self, rate: u32) {
    self.shared.set_rate(rate);
  }

  /// Pushes the block starting at mixer frame `frame`, its stereo pairs `samples`, while the stream runs.
  pub(crate) fn push(&mut self, frame: u64, samples: &[f32]) {
    if self.shared.get_end().is_some() {
      return;
    }

    let mut block: AudioTapBlock = AudioTapBlock {
      frame,
      samples: [0.0; AudioTapBlock::SAMPLES],
    };

    block.samples.copy_from_slice(samples);

    if self.blocks.push(block).is_err() {
      self.shared.add_dropped();
    }
  }

  /// Ends the stream where the mixer goes on at a `rate` other than the tap's.
  pub(crate) fn follow_rate(&self, rate: u32) {
    if rate != self.shared.get_rate() {
      self.shared.end_at_rate(rate);
    }
  }

  /// Whether the reader has let the tap go.
  pub(crate) fn is_abandoned(&self) -> bool {
    self.blocks.is_abandoned()
  }
}
