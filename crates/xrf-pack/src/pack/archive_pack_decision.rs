//! The decisions one packing run makes about a file, named so the run can say them and count them the same way.
//!
//! Finer than the counts a result carries: a reverted or an empty entry is counted as stored, because that is what the
//! volume holds, while the line about it says why it was not compressed. [`crate::pack::ArchivePackNarrator`] says
//! them and [`crate::pack::ArchivePackResult::record_outcome`] counts them, so neither can invent a sixth answer.

/// How one file reached the archive.
///
/// Aliasing carries the entry it points at, because a placement is only worth saying with the source that explains it,
/// and because a variant nobody can construct without that source cannot be reported without it either.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchivePackEntryOutcome<'e> {
  /// LZO payload, smaller than the file by more than the margin xrCompress requires.
  Compressed,
  /// Written as read, because the engine does not expect this kind of file compressed.
  Stored,
  /// Written as read after compression saved nothing worth the round trip, `VFS (R)` in xrCompress.
  Reverted,
  /// A zero-byte file, which has nothing to compress.
  Empty,
  /// A descriptor row pointing at an identical payload already in this volume, written for `source`.
  Aliased { source: &'e str },
}

/// Which rule kept a file out of the archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchivePackSkipReason {
  /// Matched a configured `exclude_exts` pattern.
  ExcludedExtension,
  /// Matched the built-in list of editor and source leftovers the engine build drops.
  SkipList,
}

impl ArchivePackSkipReason {
  /// The parenthesised explanation a transcript line carries.
  pub(crate) const fn as_label(self) -> &'static str {
    match self {
      Self::ExcludedExtension => "excluded extension",
      Self::SkipList => "skip list",
    }
  }
}
