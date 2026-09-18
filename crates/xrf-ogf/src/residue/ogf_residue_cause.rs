/// Why residue is inert to the engine, which is the only reason it is tolerated at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OgfResidueCause {
  /// Too few bytes to be a chunk header.
  TrailingFragment,
  /// Completes a motion reference path the declared size of chunk 24 or 19 cut in half.
  SplitMotionRef { path: String },
}

impl OgfResidueCause {
  /// Stable identifier for reports: `split-motion-ref` or `trailing-fragment`.
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::TrailingFragment => "trailing-fragment",
      Self::SplitMotionRef { .. } => "split-motion-ref",
    }
  }

  /// The motion reference path normalizing discards, for the split-reference shape only.
  pub fn get_discarded_path(&self) -> Option<&str> {
    match self {
      Self::SplitMotionRef { path } => Some(path),
      Self::TrailingFragment => None,
    }
  }
}
