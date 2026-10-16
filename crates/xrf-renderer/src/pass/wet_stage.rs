/// The wet surfaces' two stages, in the order they draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WetStage {
  /// The patches, where the surfaces are wet.
  Patch,
  /// The wet look written back over the G-buffer.
  Apply,
}

impl WetStage {
  pub const ALL: [Self; 2] = [Self::Patch, Self::Apply];
  pub const COUNT: usize = Self::ALL.len();

  /// Its place among the stages, which its pipeline is kept at.
  pub const fn get_index(self) -> usize {
    self as usize
  }
}
