//! Where one packing run reads the payloads of the entries it was given.

use std::fs;

use xrf_error::XrfResult;
use xrf_vfs::XraySearch;

use crate::pack::source::{ArchivePackEntry, ArchivePackOrigin};

/// The one place a run's payloads come from.
pub(crate) enum ArchivePackPayloads<'a> {
  /// The host filesystem, at the path each entry's walk reached it by.
  Host,
  /// A search, under the engine identity that is also each entry's name.
  Mounted(&'a XraySearch),
}

impl ArchivePackPayloads<'_> {
  /// Read one entry's bytes.
  ///
  /// # Errors
  ///
  /// Returns the read error of the host file or the mounted entry, and an unexpected error where an entry's origin
  /// does not match the run's source — which registration makes unreachable and is reported rather than ignored.
  pub(crate) fn read(&self, entry: &ArchivePackEntry) -> XrfResult<Vec<u8>> {
    match (self, &entry.origin) {
      (Self::Host, ArchivePackOrigin::Host(path)) => Ok(fs::read(path)?),
      (Self::Mounted(search), ArchivePackOrigin::Mounted) => search.read(&entry.name),
      _ => Err(xrf_error::XrfError::new_unexpected_error(format!(
        "Packing entry '{}' came from a source this run does not read from",
        entry.name
      ))),
    }
  }
}
