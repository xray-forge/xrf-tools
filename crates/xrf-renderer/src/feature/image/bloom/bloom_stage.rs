/// The engine's bloom's three stages, in the order they draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BloomStage {
  /// Built from the high target.
  Build,
  /// Blurred across.
  Across,
  /// Blurred down.
  Down,
}

impl BloomStage {
  pub const ALL: [Self; 3] = [Self::Build, Self::Across, Self::Down];
  pub const COUNT: usize = Self::ALL.len();

  /// Its place among the stages, which its pipeline is kept at.
  pub const fn get_index(self) -> usize {
    self as usize
  }

  /// The stage's pass, as the frame graph names it.
  pub const fn get_name(self) -> &'static str {
    match self {
      Self::Build => "bloom build",
      Self::Across => "bloom across",
      Self::Down => "bloom down",
    }
  }
}
