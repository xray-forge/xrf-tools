/// Which of the graph's mechanisms a compile applies; each can be turned off alone to bisect a difference in a capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphCompileOptions {
  /// Drops passes whose effects nothing reads.
  pub is_culling: bool,
  /// Lets transients whose lifetimes do not overlap share one texture or buffer.
  pub is_pooling: bool,
  /// Draws consecutive raster passes into the same attachments in one render pass.
  pub is_merging: bool,
  /// Records each encode group into an encoder of its own; off, the whole frame is one.
  pub is_grouping: bool,
}

impl Default for GraphCompileOptions {
  fn default() -> Self {
    Self {
      is_culling: true,
      is_pooling: true,
      is_merging: true,
      is_grouping: true,
    }
  }
}

impl GraphCompileOptions {
  /// Every mechanism off: each pass declared runs, in its own render or compute pass, its transients apart, in one
  /// encoder.
  pub fn serial() -> Self {
    Self {
      is_culling: false,
      is_pooling: false,
      is_merging: false,
      is_grouping: false,
    }
  }
}
