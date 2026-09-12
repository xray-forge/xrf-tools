use xrf_error::{XrfError, XrfResult};

use crate::ArchiveProject;
use crate::archive_file_descriptor::ArchiveFileDescriptor;
use crate::project::archive_read_result::ArchiveReadResult;

impl ArchiveProject {
  /// Read one archived file into memory, decompressing it when it is stored compressed.
  ///
  /// A query about what the project holds rather than an unpacking step: nothing reaches the filesystem
  /// beyond the archive itself. Callers that need the bytes have to hold them, so any size limit belongs
  /// with the caller; [`Self::read_file_as_string`] applies the project's read policy for that reason.
  pub fn read_file_bytes(&self, name: &str) -> XrfResult<Vec<u8>> {
    let descriptor: &ArchiveFileDescriptor = self
      .files
      .get(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Cannot read '{name}' - no such file in the archive.")))?;

    // One read, so one volume opened: the same seam an unpack holds across tens of thousands of entries, used here for
    // the single entry this asks for.
    self.open_volumes()?.read_bytes(descriptor)
  }

  /// Read one archived file as text, subject to the project's read policy.
  ///
  /// Unlike [`Self::read_file_bytes`], the extension and size gates apply: this exists for a viewer that shows a config,
  /// so refusing a binary or an enormous entry by name is the point rather than a limitation.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry is absent, the policy refuses it, or its bytes cannot be read or decoded.
  pub fn read_file_as_string(&self, filename: &str) -> XrfResult<ArchiveReadResult> {
    log::info!("Trying to read file from archive: {filename}");

    let descriptor: &ArchiveFileDescriptor = self
      .files
      .get(filename)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{filename}' is not found in the archive project")))?;

    self.read_policy.require_text_read(filename, descriptor.size_real)?;

    ArchiveReadResult::decode(filename, &self.read_file_bytes(filename)?, descriptor.size_real)
  }
}
