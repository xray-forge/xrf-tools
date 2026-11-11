use std::path::{Path, PathBuf};

use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::scene::level_scene_decompile::LevelSceneTextureFile;

/// The textures a build merges into its atlases, the detail objects' and the impostors' strips: those held in memory
/// (a decompile's own, by name) before those under the textures root (`$game_textures$`), each as `Surface_Load` reads
/// its `.dds`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LevelBuildTextures<'a> {
  root: Option<&'a Path>,
  held: &'a [LevelSceneTextureFile],
}

impl<'a> LevelBuildTextures<'a> {
  pub(crate) fn new(root: Option<&'a Path>, held: &'a [LevelSceneTextureFile]) -> Self {
    Self { root, held }
  }

  /// Whether any texture can be read: a root is named or some are held.
  pub(crate) fn is_available(&self) -> bool {
    self.root.is_some() || !self.held.is_empty()
  }

  /// The top level of texture `name`, a texture path such as `lod\lod_tree`, held or under the root; `kind` names what
  /// it is for in an error.
  ///
  /// # Errors
  ///
  /// Returns an error when no held texture has the name and the root names none, or the texture does not decode.
  pub(crate) fn read(&self, kind: &str, name: &str) -> XrfResult<RgbaImage> {
    if let Some(held) = self.held.iter().find(|held| held.name.eq_ignore_ascii_case(name)) {
      return DdsFile::read_from_bytes(&held.bytes)
        .and_then(|file| file.decode_rgba(0))
        .map_err(|error| XrfError::new_read_error(format!("{kind} '{name}' held in memory does not read: {error}")));
    }

    let root: &Path = self.root.ok_or_else(|| {
      XrfError::new_invalid_error(format!("{kind} '{name}' is not held and no textures root is named"))
    })?;
    let path: PathBuf = LevelSceneTextureFile::get_named_path(root, name);

    DdsFile::read_from_path(&path)
      .and_then(|file| file.decode_rgba(0))
      .map_err(|error| {
        XrfError::new_read_error(format!(
          "{kind} '{name}' was not read from {}: {error}",
          format_path(&path)
        ))
      })
  }
}
