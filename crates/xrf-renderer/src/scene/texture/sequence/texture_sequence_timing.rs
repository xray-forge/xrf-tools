/// When each frame of a sequence shows (`CTexture::apply_seq`): a frame every `frame_ms` of the engine's continual
/// clock, looping, or back and forth where the sequence is cycled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextureSequenceTiming {
  /// Whole milliseconds a frame shows, `1000 / fps` as the engine divides it (`seqMSPF`).
  pub frame_ms: u32,
  /// Whether it plays forward then back (`seqCycles`), each end frame shown twice in a row.
  pub is_cycled: bool,
}

impl TextureSequenceTiming {
  /// The frame of `count` shown `elapsed_ms` into the clock.
  pub(crate) fn get_frame(self, elapsed_ms: u64, count: u32) -> u32 {
    let count: u64 = u64::from(count.max(1));
    let frame: u64 = elapsed_ms / u64::from(self.frame_ms.max(1));

    let shown: u64 = if self.is_cycled {
      let id: u64 = frame % (count * 2);

      if id >= count { count - 1 - id % count } else { id }
    } else {
      frame % count
    };

    shown as u32
  }
}
