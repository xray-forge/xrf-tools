/// SMAA's three stages, in the order they draw.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmaaStage {
  /// The edges found in the frame.
  Edges,
  /// The blend weights along those edges.
  Weights,
  /// The frame blended by its weights.
  Blend,
}

impl SmaaStage {
  pub const ALL: [Self; 3] = [Self::Edges, Self::Weights, Self::Blend];
  pub const COUNT: usize = Self::ALL.len();

  /// Its place among the stages, which its pipeline is kept at.
  pub const fn get_index(self) -> usize {
    self as usize
  }

  /// The stage's pass, as the frame graph names it.
  pub const fn get_name(self) -> &'static str {
    match self {
      Self::Edges => "smaa edges",
      Self::Weights => "smaa weights",
      Self::Blend => "smaa",
    }
  }
}
