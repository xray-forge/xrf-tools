/// Why residue is inert to the engine, which is the only reason it is tolerated at all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OgfResidueCause {
  /// Too few bytes to be a chunk header.
  ///
  /// The engine's own walk reads four bytes here regardless, two of them past the buffer, because
  /// `VERIFY(Pos + cnt <= Size)` (`xray-16/src/xrCore/FS.cpp:367`) is debug-only. Tolerated because shipped assets
  /// carry it, not because it is correct.
  TrailingFragment,
  /// Completes a motion reference path the declared size of chunk 24 or 19 cut in half.
  ///
  /// The count-governed read never reaches it, so the path is never loaded. Reported because normalizing discards it
  /// and this is the only record that it was there.
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
