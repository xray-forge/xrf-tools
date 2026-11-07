/// Why a departure from a well-formed chunk tree is invisible to the engine, which is the only reason it is tolerated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OgfResidueCause {
  /// Too few bytes to be a chunk header.
  TrailingFragment,
  /// Completes a motion reference path the declared size of chunk 24 or 19 cut in half, past the count the loader reads.
  SplitMotionRef { path: String },
  /// The counted motion references of chunk 24 or 19 run past its declared size; the loader's `r_stringZ` reads them
  /// on from the file, so they are part of the chunk and only its size understates them.
  UnderstatedMotionRefs { declared: u32 },
  /// A last motion refs chunk declares more bytes than the file holds, but its references end exactly where the file
  /// does, and the loader reads them off the visual's own reader whatever the size says.
  OverstatedMotionRefs { declared: u32 },
  /// A last LODs chunk declares more bytes than the file holds, but its line ends within the file, where the loader's
  /// `r_string` stops reading, past one more byte it looks at for a further line end.
  OverstatedLods { declared: u32 },
  /// Bytes after the last chunk, headed by an id no loader asks for and a size past the file's end: `find_chunk` skips
  /// them by that size and ends its search, so they are never read. Text appended to a visual reads so.
  UnreadTrailingChunk { id: u32 },
}

impl OgfResidueCause {
  /// Stable identifier for reports.
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::TrailingFragment => "trailing-fragment",
      Self::SplitMotionRef { .. } => "split-motion-ref",
      Self::UnderstatedMotionRefs { .. } => "understated-motion-refs",
      Self::OverstatedMotionRefs { .. } => "overstated-motion-refs",
      Self::OverstatedLods { .. } => "overstated-lods",
      Self::UnreadTrailingChunk { .. } => "unread-trailing-chunk",
    }
  }

  /// The motion reference path normalizing discards, for the split-reference shape only.
  pub fn get_discarded_path(&self) -> Option<&str> {
    match self {
      Self::SplitMotionRef { path } => Some(path),
      Self::TrailingFragment
      | Self::UnderstatedMotionRefs { .. }
      | Self::OverstatedMotionRefs { .. }
      | Self::OverstatedLods { .. }
      | Self::UnreadTrailingChunk { .. } => None,
    }
  }
}
