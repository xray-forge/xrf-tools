/// A published value presentation draws between two ticks.
pub trait LoopBlend: Copy {
  /// The value `alpha` of the way from this one to `next`, `0` this one and `1` the next.
  fn blend(&self, next: &Self, alpha: f32) -> Self;
}
