/// A published value presentation draws between two ticks.
pub trait LoopBlend {
  /// The value `alpha` of the way from this one to `next`, `0` this one and `1` the next.
  #[must_use]
  fn blend(&self, next: &Self, alpha: f32) -> Self;
}
