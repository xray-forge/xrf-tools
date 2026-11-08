/// The wet surfaces' two stages, in the order they draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WetStage {
  /// The patches, where the surfaces are wet.
  Patch,
  /// The wet look written back over the G-buffer.
  Apply,
}

impl WetStage {
  /// Its place among the stages, which its pipeline is kept at.
  pub(crate) const fn get_index(self) -> usize {
    self as usize
  }
}
