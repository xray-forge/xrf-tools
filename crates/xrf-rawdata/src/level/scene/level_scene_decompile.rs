use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_level::EditableLevelScene;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostDestination, HostLinkPolicy, HostRelativePath, StagedFile, format_path};

use crate::level::scene::level_scene_library_file::LevelSceneLibraryFile;
use crate::level::scene::level_scene_object_file::LevelSceneObjectFile;
use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;
use crate::level::scene::level_scene_write::LevelSceneWrite;
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

  /// [`Self::write_to_path_with`], following any link below the roots as the filesystem resolves it.
  ///
  /// # Errors
  ///
  /// As [`Self::write_to_path_with`].
  pub fn write_to_path<T: ByteOrder>(&self, level_path: &Path, objects_root: &Path, textures_root: &Path) -> XrfResult {
    self.write_to_path_with::<T>(level_path, objects_root, textures_root, HostLinkPolicy::Follow)
  }

  /// Writes the decompile: the scene's sidecar, each object after its own sidecar, the library copies (skipping files
  /// that already hold them), the textures but a library object's own the root holds, then the scene at `level_path`
  /// with its parts beside it.
  ///
  /// Each file goes through a [`HostDestination`] at its root (the scene's directory, the objects root, the textures
  /// root) under `link_policy`. Each sidecar goes before what it describes, so a failed write leaves a stray sidecar,
  /// never a decompile taken for authored work.
  ///
  /// # Errors
  ///
  /// Returns an error, writing nothing, when [`EditableLevelScene::check_writable`] refuses the scene, a name from data
  /// leaves its root, or a refused link already stands where a file goes; otherwise when a file cannot be written, as
  /// [`EditableLevelScene::write_to_path`] does.
  pub fn write_to_path_with<T: ByteOrder>(
    &self,
    level_path: &Path,
    objects_root: &Path,
    textures_root: &Path,
    link_policy: HostLinkPolicy,
  ) -> XrfResult {
    let write: LevelSceneWrite<'_> = self.check_write(level_path, objects_root, textures_root, link_policy)?;

    self.write_checked::<T>(write)
  }

  /// Resolves every path a write writes and checks each against its root's link policy, creating nothing, so a caller
  /// can refuse before it changes anything on disk. Whether the scene's part directory holds parts it would not write
  /// is left to [`Self::write_checked`].
  ///
  /// # Errors
  ///
  /// Returns an error when a name from data leaves its root, or a refused link already stands where a file goes.
  pub(crate) fn check_write(
    &self,
    level_path: &Path,
    objects_root: &Path,
    textures_root: &Path,
    link_policy: HostLinkPolicy,
  ) -> XrfResult<LevelSceneWrite<'_>> {
    let scene_root: HostDestination =
      HostDestination::new(level_path.parent().unwrap_or_else(|| Path::new("")), link_policy);
    let objects_root: HostDestination = HostDestination::new(objects_root, link_policy);
    let textures_root: HostDestination = HostDestination::new(textures_root, link_policy);
    let scene: Vec<PathBuf> = self.get_written_scene_paths(level_path)?;
    let objects: Vec<PathBuf> = self
      .objects
      .iter()
      .map(|object| object.get_path(objects_root.get_root()))
      .collect::<XrfResult<_>>()?;
    let library: Vec<(PathBuf, PathBuf)> = self
      .library
      .iter()
      .map(|file| file.get_paths(objects_root.get_root()))
      .collect::<XrfResult<_>>()?;
    let textures: Vec<(&LevelSceneTextureFile, PathBuf)> = self.get_written_textures(textures_root.get_root())?;

    for path in &scene {
      Self::check_file(&scene_root, path)?;
    }

    for path in &objects {
      Self::check_file(&objects_root, &ObjectProvenance::sidecar_path(path))?;
      Self::check_file(&objects_root, path)?;
    }

    for (path, thumbnail) in &library {
      Self::check_file(&objects_root, path)?;
      Self::check_file(&objects_root, thumbnail)?;
    }

    for (_, path) in &textures {
      Self::check_file(&textures_root, path)?;
    }

    Ok(LevelSceneWrite {
      level_path: level_path.to_path_buf(),
      scene_root,
      objects_root,
      textures_root,
      scene,
      objects,
      library,
      textures,
    })
  }

  /// Carries out `write`, which [`Self::check_write`] made of this decompile, once
  /// [`EditableLevelScene::check_writable`] accepts the scene.
  ///
  /// # Errors
  ///
  /// Returns an error, writing nothing, when [`EditableLevelScene::check_writable`] refuses the scene; otherwise when a
  /// file cannot be written, as [`EditableLevelScene::write_to_path`] does.
  pub(crate) fn write_checked<T: ByteOrder>(&self, write: LevelSceneWrite<'_>) -> XrfResult {
    let LevelSceneWrite {
      level_path,
      scene_root,
      objects_root,
      textures_root,
      scene,
      objects,
      library,
      textures,
    } = write;
    let level_path: &Path = &level_path;

    // Refused before the sidecar is written, so a sidecar never lands beside an authored scene left as it was. Checked
    // here rather than with the paths: a forced decompile removes an earlier scene's stale parts between the two.
    self.scene.check_writable(level_path)?;

    Self::write_file(
      &scene_root,
      &LevelSceneProvenance::sidecar_path(level_path),
      self.provenance.to_json()?.as_bytes(),
    )?;

    for (object, path) in self.objects.iter().zip(&objects) {
      Self::write_file(
        &objects_root,
        &ObjectProvenance::sidecar_path(path),
        object.provenance.to_json()?.as_bytes(),
      )?;
      Self::write_file(&objects_root, path, &object.object.write_to_bytes::<T>()?)?;
    }

    for (file, (path, thumbnail)) in self.library.iter().zip(library) {
      if !Self::is_same_file(&path, &file.object) {
        Self::write_file(&objects_root, &path, &file.object)?;
      }

      if let Some(bytes) = file
        .thumbnail
        .as_deref()
        .filter(|bytes| !Self::is_same_file(&thumbnail, bytes))
      {
        Self::write_file(&objects_root, &thumbnail, bytes)?;
      }
    }

    for (texture, path) in textures {
      Self::write_file(&textures_root, &path, &texture.bytes)?;
    }

    // The scene writes its own files, into the directories made and through the slots checked here.
    scene_root.create_root()?;

    for path in &scene {
      scene_root.prepare_file(&Self::get_relative(&scene_root, path)?)?;
    }

    self.scene.write_to_path::<T, _>(&level_path)
  }

  /// `path`, which a getter joined below `destination`'s root, as the path below it.
  fn get_relative(destination: &HostDestination, path: &Path) -> XrfResult<HostRelativePath> {
    let relative: Option<&str> = path.strip_prefix(destination.get_root()).ok().and_then(Path::to_str);

    match relative {
      Some(relative) => HostRelativePath::new_file(relative, "Decompiled file"),
      None => Err(XrfError::new_invalid_error(format!(
        "{} does not lie below {}",
        format_path(path),
        format_path(destination.get_root())
      ))),
    }
  }

  /// Checks `path` against `destination`'s link policy as far as it exists, creating nothing.
  fn check_file(destination: &HostDestination, path: &Path) -> XrfResult {
    destination.resolve_file(&Self::get_relative(destination, path)?)?;

    Ok(())
  }

  /// Writes `bytes` at `path` below `destination`, creating the root where this is its first file: a root is made only
  /// where something is written into it.
  fn write_file(destination: &HostDestination, path: &Path, bytes: &[u8]) -> XrfResult {
    let relative: HostRelativePath = Self::get_relative(destination, path)?;

    destination.create_root()?;

    let path: PathBuf = destination.prepare_file(&relative)?;

    StagedFile::write(&path, bytes)
      .map_err(|error| XrfError::new_io_error(format!("{} was not written: {error}", format_path(&path)), error.kind()))
  }
}
