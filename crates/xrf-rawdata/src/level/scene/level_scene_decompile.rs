use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_level::EditableLevelScene;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::level::scene::level_scene_library_file::LevelSceneLibraryFile;
use crate::level::scene::level_scene_object_file::LevelSceneObjectFile;
use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;
use crate::provenance::level_scene_provenance::LevelSceneProvenance;
use crate::provenance::object_provenance::ObjectProvenance;

/// A decompiled level: its scene, the provenance written beside it, the objects it places, and the textures they need.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelSceneDecompile {
  pub scene: EditableLevelScene,
  pub provenance: LevelSceneProvenance,
  pub objects: Vec<LevelSceneObjectFile>,
  /// The library objects it places by reference, copied beside the objects.
  pub library: Vec<LevelSceneLibraryFile>,
  pub textures: Vec<LevelSceneTextureFile>,
}

impl LevelSceneDecompile {
  /// What a write would replace of the textures the objects need, under `textures_root`; a library object's own
  /// texture the root holds is kept, so replaces nothing.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when a texture's name names no file below the root.
  pub fn get_existing_texture_paths(&self, textures_root: &Path) -> XrfResult<Vec<PathBuf>> {
    Ok(
      self
        .get_written_textures(textures_root)?
        .into_iter()
        .map(|(_, path)| path)
        .filter(|path| path.exists())
        .collect(),
    )
  }

  /// The textures a write puts under `textures_root`, each with its path: all but a library object's own the root
  /// already holds.
  fn get_written_textures(&self, textures_root: &Path) -> XrfResult<Vec<(&LevelSceneTextureFile, PathBuf)>> {
    let mut written: Vec<(&LevelSceneTextureFile, PathBuf)> = Vec::new();

    for texture in &self.textures {
      let path: PathBuf = texture.get_path(textures_root)?;

      if !texture.is_held_at(&path) {
        written.push((texture, path));
      }
    }

    Ok(written)
  }

  /// What a write would replace of the objects the scene places, under `objects_root`, and their sidecars; a library
  /// copy that would write a file's own bytes again replaces nothing, as where the objects root is the library's.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when an object's reference names no file below the root.
  pub fn get_existing_object_paths(&self, objects_root: &Path) -> XrfResult<Vec<PathBuf>> {
    let mut written: Vec<(PathBuf, Option<&[u8]>)> = Vec::new();

    for path in self.get_written_object_paths(objects_root)? {
      written.push((path, None));
    }

    for file in &self.library {
      let (path, thumbnail) = file.get_paths(objects_root)?;

      written.push((path, Some(file.object.as_slice())));
      written.push((thumbnail, file.thumbnail.as_deref()));
    }

    Ok(
      written
        .into_iter()
        .filter(|(path, bytes)| path.exists() && !bytes.is_some_and(|bytes| Self::is_same_file(path, bytes)))
        .map(|(path, _)| path)
        .collect(),
    )
  }

  /// Whether the file at `path` holds `bytes`.
  fn is_same_file(path: &Path, bytes: &[u8]) -> bool {
    std::fs::read(path).is_ok_and(|held| held == bytes)
  }

  /// What a write to `level_path` writes of the scene: its main file, its sidecar and each of its parts.
  ///
  /// # Errors
  ///
  /// As [`EditableLevelScene::get_part_paths`].
  pub fn get_written_scene_paths(&self, level_path: &Path) -> XrfResult<Vec<PathBuf>> {
    Ok(
      [level_path.to_path_buf(), LevelSceneProvenance::sidecar_path(level_path)]
        .into_iter()
        .chain(self.scene.get_part_paths(level_path)?)
        .collect(),
    )
  }

  /// What a write writes of the level's own objects under `objects_root`: each object's sidecar, then the object.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when an object's reference names no file below the root.
  pub fn get_written_object_paths(&self, objects_root: &Path) -> XrfResult<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();

    for object in &self.objects {
      let path: PathBuf = object.get_path(objects_root)?;

      paths.push(ObjectProvenance::sidecar_path(&path));
      paths.push(path);
    }

    Ok(paths)
  }

  /// Writes the decompile: the scene's sidecar, each object after its own sidecar, the library copies (skipping files
  /// that already hold them), the textures but a library object's own the root holds, then the scene at `level_path`
  /// with its parts beside it.
  ///
  /// Each sidecar goes before what it describes, so a failed write leaves a stray sidecar, never a decompile taken for
  /// authored work.
  ///
  /// # Errors
  ///
  /// Returns an error, writing nothing, when [`EditableLevelScene::check_writable`] refuses the scene or a name from
  /// data leaves its root; otherwise when a file cannot be written, as [`EditableLevelScene::write_to_path`] does.
  pub fn write_to_path<T: ByteOrder>(&self, level_path: &Path, objects_root: &Path, textures_root: &Path) -> XrfResult {
    // Refused before the sidecar is written, so a sidecar never lands beside an authored scene left as it was.
    self.scene.check_writable(level_path)?;

    // Every path resolved before the first write, so a name that would leave its root writes nothing.
    let objects: Vec<PathBuf> = self
      .objects
      .iter()
      .map(|object| object.get_path(objects_root))
      .collect::<XrfResult<_>>()?;
    let library: Vec<(PathBuf, PathBuf)> = self
      .library
      .iter()
      .map(|file| file.get_paths(objects_root))
      .collect::<XrfResult<_>>()?;
    let textures: Vec<(&LevelSceneTextureFile, PathBuf)> = self.get_written_textures(textures_root)?;

    Self::write_file(
      &LevelSceneProvenance::sidecar_path(level_path),
      self.provenance.to_json()?.as_bytes(),
    )?;

    for (object, path) in self.objects.iter().zip(&objects) {
      Self::write_file(
        &ObjectProvenance::sidecar_path(path),
        object.provenance.to_json()?.as_bytes(),
      )?;
      Self::write_file(path, &object.object.write_to_bytes::<T>()?)?;
    }

    for (file, (path, thumbnail)) in self.library.iter().zip(library) {
      if !Self::is_same_file(&path, &file.object) {
        Self::write_file(&path, &file.object)?;
      }

      if let Some(bytes) = file
        .thumbnail
        .as_deref()
        .filter(|bytes| !Self::is_same_file(&thumbnail, bytes))
      {
        Self::write_file(&thumbnail, bytes)?;
      }
    }

    for (texture, path) in textures {
      Self::write_file(&path, &texture.bytes)?;
    }

    self.scene.write_to_path::<T, _>(&level_path)
  }

  fn write_file(path: &Path, bytes: &[u8]) -> XrfResult {
    StagedFile::write_creating_directories(path, bytes)
      .map_err(|error| XrfError::new_io_error(format!("{} was not written: {error}", format_path(path)), error.kind()))
  }
}
