/// How much of the rain one frame draws.
#[derive(Clone, Copy, Debug)]
pub struct RainCounts {
  /// The streaks drawn in all.
  pub streaks: u32,
  /// The first of them, which fall in the near square and alone ring and splash.
  pub near: u32,
  /// The splash model's indices.
  pub splash_indices: u32,
}
