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
