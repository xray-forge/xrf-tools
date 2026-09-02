use xrf_error::{XrfError, XrfResult};
use xrf_utils::encode_w1251_bytes_to_string;

use crate::ArchiveProject;
use crate::archive_file_descriptor::ArchiveFileDescriptor;
use crate::project::archive_project_read_result::ProjectReadResult;

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
  /// Returns an error when the extension is not one the policy reads as text, the entry is absent, it exceeds the
  /// policy's size limit, or its bytes cannot be read or decoded.
  pub fn read_file_as_string(&self, filename: &str) -> XrfResult<ProjectReadResult> {
    log::info!("Trying to read file from archive: {filename}");

    if !self.read_policy.supports_file(filename) {
      return Err(XrfError::new_read_error(format!(
        "File '{filename}' cannot be read, file extension is not allowed to be read"
      )));
    }

    let descriptor: &ArchiveFileDescriptor = self
      .files
      .get(filename)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{filename}' is not found in the archive project")))?;

    if descriptor.size_real > self.read_policy.maximum_size {
      return Err(XrfError::new_read_error(format!(
        "File '{filename}' is too big to be read - {}, {} is maximum allowed",
        descriptor.size_real, self.read_policy.maximum_size
      )));
    }

    let bytes: Vec<u8> = self.read_file_bytes(filename)?;

    // Archive text is Windows-1251, like every engine text format: a lossy UTF-8 read here turned Cyrillic configs
    // into replacement characters in the archive explorer.
    Ok(ProjectReadResult::new(
      filename,
      &encode_w1251_bytes_to_string(&bytes)?,
      descriptor.size_real,
    ))
  }
}
