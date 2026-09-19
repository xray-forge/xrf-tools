use std::fs::File;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};

use crate::payload::archive_located_entry::ArchiveLocatedEntry;
use crate::{ArchiveDescriptor, ArchiveFileDescriptor};

/// One open volume, with the length every entry of it is bounded by.
pub(super) struct ArchiveOpenVolume {
  file: File,
  size: u64,
}

impl ArchiveOpenVolume {
  pub(super) fn open(path: &Path) -> XrfResult<Self> {
    let file: File = File::open(path)?;
    let size: u64 = file.metadata()?.len();

    Ok(Self { file, size })
  }

  /// Locates an entry only after its stored byte range is proven to fit this volume.
  pub(super) fn locate<'a>(
    &'a self,
    volume: &'a ArchiveDescriptor,
    descriptor: &'a ArchiveFileDescriptor,
  ) -> XrfResult<ArchiveLocatedEntry<'a>> {
    let offset: u64 = u64::from(descriptor.offset);
    let end: u64 = offset + u64::from(descriptor.size_compressed);

    if end > self.size {
      return Err(XrfError::new_read_error(format!(
        "entry '{}' declares bytes {}..{end}, beyond its volume's {}-byte end",
        descriptor.name, descriptor.offset, self.size
      )));
    }

    Ok(ArchiveLocatedEntry {
      file: &self.file,
      offset,
      volume,
      descriptor,
    })
  }
}
