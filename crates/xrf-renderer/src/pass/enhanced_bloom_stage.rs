/// The enhanced bloom's stages, by the pipeline each draws with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnhancedBloomStage {
  /// A blur of the frame at half its size, across or down.
  Blur,
  /// The bloom built from the blurred frame.
  Build,
  /// A halving of the bloom.
  Halve,
  /// A doubling of the bloom, adding the halving of its size.
  Double,
  /// The bloom finished at half the frame's size.
  Finish,
}

impl EnhancedBloomStage {
  pub const ALL: [Self; 5] = [Self::Blur, Self::Build, Self::Halve, Self::Double, Self::Finish];
  pub const COUNT: usize = Self::ALL.len();

  /// Its place among the stages, which its pipeline is kept at.
  pub const fn get_index(self) -> usize {
    self as usize
  }
}
