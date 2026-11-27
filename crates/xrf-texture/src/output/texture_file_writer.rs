use std::path::Path;

use xrf_dds::DdsFile;
use xrf_error::XrfResult;

/// Where a processor puts each texture file it writes, once the file is whole: written at its path, or staged for its
/// caller to publish with whatever else it writes, so a pair lands together or not at all.
pub trait TextureFileWriter {
  /// Takes `file`, to stand at `path` once written.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be written or staged.
  fn write(&mut self, path: &Path, file: &DdsFile) -> XrfResult;
}
