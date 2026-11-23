use std::time::Instant;

use crate::scene::texture::texture_cache::TextureCache;

impl TextureCache {
  /// Shows each sequence's frame due at `now`, the frames' clock, by copying it into its slot's texture on `encoder`,
  /// which must be submitted before the frame's draws; a sequence whose frame stands copies nothing.
  pub(crate) fn play(&mut self, encoder: &mut wgpu::CommandEncoder, now: Instant) {
    if self.sequences.is_empty() {
      return;
    }

    // Whole milliseconds, as the engine's continual clock counts them.
    let elapsed_ms: u64 = now.saturating_duration_since(self.started).as_millis() as u64;

    for sequence in self.sequences.values_mut() {
      sequence.play(encoder, elapsed_ms);
    }
  }

  /// The frame a slot's sequence shows, `None` for a slot holding no sequence.
  #[cfg(test)]
  pub(crate) fn get_sequence_frame(&self, slot: u32) -> Option<u32> {
    self.sequences.get(&slot).map(|sequence| sequence.get_current())
  }

  /// When the sequences' clock began, which a frame's time is counted from.
  #[cfg(test)]
  pub(crate) fn get_sequences_start(&self) -> Instant {
    self.started
  }
}
