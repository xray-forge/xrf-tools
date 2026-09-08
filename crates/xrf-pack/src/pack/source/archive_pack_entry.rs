use std::path::PathBuf;

/// Where one selected entry's bytes are, in the terms its own run reads them by.
#[derive(Clone, Debug)]
pub(crate) enum ArchivePackOrigin {
  /// A file on the host filesystem, at the path the source walk reached it by.
  Host(PathBuf),
  /// An entry of the mounted world this run compares, under its own name.
  Mounted,
}

/// One file selected for packing: the name it goes in under, and where its bytes are now.
///
/// Produced by a source and consumed by the volume writer, so it outlives both and belongs to neither.
#[derive(Clone, Debug)]
pub(crate) struct ArchivePackEntry {
  /// Name as authored: relative to the source root, with X-Ray separators. The engine folds its case on registration.
  pub(crate) name: String,
  pub(crate) origin: ArchivePackOrigin,
}

impl ArchivePackEntry {
  /// An entry the source walk found on the host.
  pub(crate) fn of_host(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
    Self {
      name: name.into(),
      origin: ArchivePackOrigin::Host(path.into()),
    }
  }

  /// An entry a mounted world answers for, named by the engine identity it answers to.
  pub(crate) fn of_mounted(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      origin: ArchivePackOrigin::Mounted,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::mem::size_of;
  use std::path::PathBuf;

  use super::{ArchivePackEntry, ArchivePackOrigin};

  #[test]
  fn an_origin_costs_no_more_than_the_host_path_it_replaced() {
    // The walk builds one of these per file and a real tree holds hundreds of thousands. `Mounted` carries nothing,
    // so the niche in `PathBuf`'s pointer holds the discriminant and the enum stays the width of the path alone.
    assert_eq!(size_of::<ArchivePackOrigin>(), size_of::<PathBuf>());
    assert_eq!(
      size_of::<ArchivePackEntry>(),
      size_of::<String>() + size_of::<PathBuf>()
    );
  }
}
