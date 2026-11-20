/// Where a stream starts and whether it starts over at its end.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SoundStreamStart {
  /// The source frame the first sample comes from, clamped to the sound's length.
  pub frame: u64,
  pub is_looped: bool,
}
