use std::path::Path;

use xrf_chunk::ChunkWriter;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

/// A file a compile step writes, staged so a failed write leaves the file as it was.
pub(crate) struct CompilerOutput;

impl CompilerOutput {
  /// Writes `bytes` to `path`.
  ///
  /// # Errors
  ///
  /// Returns an error naming `path` when the file is not written.
  pub(crate) fn write(path: &Path, bytes: &[u8]) -> XrfResult {
    StagedFile::write(path, bytes)
      .map_err(|error| XrfError::new_io_error(format!("{} was not written: {error}", format_path(path)), error.kind()))
  }

  /// Writes what `fill` puts in a chunk writer to `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when `fill` fails or the file is not written.
  pub(crate) fn write_chunks(path: &Path, fill: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    fill(&mut writer)?;

    Self::write(path, &writer.buffer)
  }
}
