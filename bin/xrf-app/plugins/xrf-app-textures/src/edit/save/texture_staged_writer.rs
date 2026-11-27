use std::io::Error as IoError;
use std::path::Path;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, AssetWrites};
use xrf_dds::DdsFile;
use xrf_error::{XrfError, XrfResult};
use xrf_texture::TextureFileWriter;

/// The textures plugin's [`AssetWrites`], taking each file a texture processor makes: a bump pair or a node's two files
/// are published together, landing whole or not at all. Dropped unpublished, it removes what it staged.
#[derive(Debug, Default)]
pub(crate) struct TextureStagedWriter {
  writes: AssetWrites,
}

impl TextureStagedWriter {
  /// Stages `bytes` to stand at `path`, for a file built whole in memory, as a descriptor is.
  ///
  /// # Errors
  ///
  /// As [`AssetWrites::stage_bytes`].
  pub(crate) fn stage_bytes(&mut self, path: &Path, bytes: &[u8]) -> TauriResult {
    self.writes.stage_bytes(path, bytes)
  }

  /// Publishes everything staged, answering every file written.
  ///
  /// # Errors
  ///
  /// As [`AssetWrites::publish`].
  pub(crate) fn publish(self, assets: &AssetMountState) -> TauriResult<Vec<String>> {
    Ok(
      self
        .writes
        .publish(assets, None)?
        .iter()
        .map(|path| path.display().to_string())
        .collect(),
    )
  }
}

impl TextureFileWriter for TextureStagedWriter {
  fn write(&mut self, path: &Path, file: &DdsFile) -> XrfResult {
    self
      .writes
      .stage_with(path, |out| file.write_to(out).map_err(IoError::other))
      .map_err(XrfError::new_texture_processing_error)
  }
}
