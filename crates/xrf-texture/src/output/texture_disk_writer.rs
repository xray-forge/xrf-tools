use std::path::Path;

use xrf_dds::DdsFile;
use xrf_error::XrfResult;

use crate::output::TextureFileWriter;

/// Writes each file at its path, staged beside it and moved into place, making the directories above it: a run with no
/// one to publish its files for it.
#[derive(Debug, Default)]
pub struct TextureDiskWriter;

impl TextureFileWriter for TextureDiskWriter {
  fn write(&mut self, path: &Path, file: &DdsFile) -> XrfResult {
    file.write_to_path(path)
  }
}
