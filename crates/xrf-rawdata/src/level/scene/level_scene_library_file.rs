use std::path::{Path, PathBuf};

use xrf_error::XrfResult;

use crate::level::scene::level_scene_object_file::LevelSceneObjectFile;

/// One library object a decompiled scene places by reference, copied as its library holds it, with the thumbnail
/// beside it where the library has one, so the scene opens and builds from the decompile's objects alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelSceneLibraryFile {
  /// Its path under the objects root without the extension, the library's own.
  pub reference: String,
  pub object: Vec<u8>,
  pub thumbnail: Option<Vec<u8>>,
}

impl LevelSceneLibraryFile {
  /// The extension of the thumbnail the Level Editor lists an object by.
  pub const THUMBNAIL_EXTENSION: &'static str = "thm";

  /// Its object's path under `objects_root`, and its thumbnail's.
  ///
  /// # Errors
  ///
  /// As [`LevelSceneObjectFile::get_reference_path`].
  pub fn get_paths(&self, objects_root: &Path) -> XrfResult<(PathBuf, PathBuf)> {
    let path: PathBuf = LevelSceneObjectFile::get_reference_path(objects_root, &self.reference)?;
    let thumbnail: PathBuf = path.with_extension(Self::THUMBNAIL_EXTENSION);

    Ok((path, thumbnail))
  }
}
