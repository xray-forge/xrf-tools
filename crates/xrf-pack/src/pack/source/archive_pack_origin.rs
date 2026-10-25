use std::path::PathBuf;

/// Where one selected entry's bytes are, in the terms its own run reads them by.
#[derive(Clone, Debug)]
pub(crate) enum ArchivePackOrigin {
  /// A file on the host filesystem, at the path the source walk reached it by.
  Host(PathBuf),
  /// An entry of the mounted world this run compares, under its own name.
  Mounted,
}
