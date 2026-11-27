use std::fs::{self, File};
use std::io::{Error as IoError, Result as IoResult, Write};
use std::path::Path;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_dds::DdsFile;
use xrf_error::{XrfError, XrfResult};
use xrf_texture::TextureFileWriter;
use xrf_utils::{StagedFile, format_path};

/// Stages each file the textures plugin writes beside its path, for the run to publish them together through
/// [`AssetMountState::write_files`]: a bump pair or a node's two files land whole or not at all, and a mounted file
/// moves the snapshot as any save does. Dropped unpublished, it removes what it staged.
#[derive(Debug, Default)]
pub(crate) struct TextureStagedWriter {
  staged: Vec<StagedFile>,
}

impl TextureStagedWriter {
  /// Stages `bytes` to stand at `path`, for a file built whole in memory, as a descriptor is.
  ///
  /// # Errors
  ///
  /// Returns an error naming `path` when its folder cannot be made or the file cannot be staged.
  pub(crate) fn stage_bytes(&mut self, path: &Path, bytes: &[u8]) -> XrfResult {
    self.stage(path, |out| out.write_all(bytes))
  }

  /// Publishes everything staged, answering every file written.
  ///
  /// # Errors
  ///
  /// Returns what [`AssetMountState::write_files`] refuses.
  pub(crate) fn publish(self, assets: &AssetMountState) -> TauriResult<Vec<String>> {
    Ok(
      assets
        .write_files(self.staged, None)?
        .iter()
        .map(|path| path.display().to_string())
        .collect(),
    )
  }

  /// Stages what `write` streams to stand at `path`, making its folder where missing, as a direct write would; the
  /// commit then places the file in it.
  fn stage(&mut self, path: &Path, write: impl FnOnce(&mut File) -> IoResult<()>) -> XrfResult {
    let staged: IoResult<StagedFile> = (|| {
      if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
      }

      StagedFile::stage_with(path, &StagedFile::name_sibling(path, "staged")?, write)
    })();

    self.staged.push(staged.map_err(|error| {
      XrfError::new_texture_processing_error(format!("Cannot stage '{}': {error}", format_path(path)))
    })?);

    Ok(())
  }
}

impl TextureFileWriter for TextureStagedWriter {
  fn write(&mut self, path: &Path, file: &DdsFile) -> XrfResult {
    self.stage(path, |out| file.write_to(out).map_err(IoError::other))
  }
}
