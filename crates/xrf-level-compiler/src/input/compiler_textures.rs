use std::path::{Path, PathBuf};

use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::HostRelativePath;

/// How the SDK's compilers name and find the textures `build.prj` names under the textures roots (`$game_textures$`).
pub(crate) struct CompilerTextures;

impl CompilerTextures {
  /// The texture the compilers take without loading its `.thm` or image, the merged impostors' atlas.
  pub(crate) const UNLOADED: &'static str = "level_lods";

  /// A texture's name as the compilers look it up: `build.prj`'s, cut at its first dot and lowered.
  pub(crate) fn get_name(name: &str) -> String {
    name.split('.').next().unwrap_or_default().to_ascii_lowercase()
  }

  /// A texture's file under a root, `<root>\<name>.<extension>`, for a name [`Self::get_name`] gave.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the name names no file below the root: one carrying a drive, or empty.
  pub(crate) fn get_path(root: &Path, name: &str, extension: &str) -> XrfResult<PathBuf> {
    Ok(root.join(HostRelativePath::new_file(name, "Texture name")?.with_extension(extension)))
  }

  /// A texture's file under the first of `roots` holding it.
  ///
  /// # Errors
  ///
  /// As [`Self::get_path`].
  pub(crate) fn find(roots: &[PathBuf], name: &str, extension: &str) -> XrfResult<Option<PathBuf>> {
    for root in roots {
      let path: PathBuf = Self::get_path(root, name, extension)?;

      if path.is_file() {
        return Ok(Some(path));
      }
    }

    Ok(None)
  }

  /// The top image of an implicitly lit texture's `.dds`, under the first of `roots` holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when no root holds the texture, or its image does not read.
  pub(crate) fn read_implicit_image(roots: &[PathBuf], name: &str) -> XrfResult<RgbaImage> {
    let path: PathBuf = Self::find(roots, name, "dds")?.ok_or_else(|| {
      XrfError::new_not_found_error(format!(
        "The implicitly lit texture {name} has no .dds under the texture roots"
      ))
    })?;

    DdsFile::read_from_path(&path)?.decode_rgba(0)
  }
}
