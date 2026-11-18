use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_utils::HostRelativePath;

/// One texture a decompiled scene's objects need for the Build, written under the game textures root
/// (`$game_textures$`), such as a tree model's impostor strip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelSceneTextureFile {
  /// Its name under the textures root without the extension, as the editor asks for it: `lod\lod_<model>`.
  pub name: String,
  /// The texture, a DDS file.
  pub bytes: Vec<u8>,
  /// Whether it stands in for a library object's own texture, such as a library tree's impostor strip cut from the
  /// level: a textures root holding the library's own keeps it.
  pub is_library: bool,
}

impl LevelSceneTextureFile {
  /// The extension the name leaves off.
  pub const EXTENSION: &'static str = "dds";

  /// Its path under `textures_root`, the name's folders as directories.
  ///
  /// # Errors
  ///
  /// As [`Self::get_named_path`].
  pub fn get_path(&self, textures_root: &Path) -> XrfResult<PathBuf> {
    Self::get_named_path(textures_root, &self.name)
  }

  /// Whether `textures_root` already holds the library's own texture this one stands in for, which a write leaves be.
  ///
  /// # Errors
  ///
  /// As [`Self::get_named_path`].
  pub fn is_held_by(&self, textures_root: &Path) -> XrfResult<bool> {
    Ok(self.is_held_at(&self.get_path(textures_root)?))
  }

  /// As [`Self::is_held_by`], for its path under the root once resolved.
  pub(super) fn is_held_at(&self, path: &Path) -> bool {
    self.is_library && path.exists()
  }

  /// The path under `textures_root` of the texture `name` names, as the editor resolves `$game_textures$`.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the name, taken from a library object's surface or a scene, names no file below the
  /// root: one climbing out of it with `..`, carrying a drive, or empty.
  pub fn get_named_path(textures_root: &Path, name: &str) -> XrfResult<PathBuf> {
    let relative: HostRelativePath = HostRelativePath::new_file(name, "Texture name")?;

    Ok(textures_root.join(relative.with_extension(Self::EXTENSION)))
  }
}
