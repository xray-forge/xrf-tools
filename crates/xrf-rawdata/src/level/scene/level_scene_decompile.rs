use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_level::EditableLevelScene;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostRelativePath, StagedFile, format_path};

use crate::provenance::level_scene_provenance::LevelSceneProvenance;
use crate::provenance::object_provenance::ObjectProvenance;

/// One object a decompiled scene places, written under the objects root (`$objects$`) with its own sidecar.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelSceneObjectFile {
  /// Its path under the objects root without the extension, as the scene object's `reference_name` names it.
  pub reference: String,
  pub object: EditableObjectFile,
  pub provenance: ObjectProvenance,
}

impl LevelSceneObjectFile {
  /// The extension the reference leaves off.
  pub const EXTENSION: &'static str = "object";

  /// Its path under `objects_root`, the reference's folders as directories.
  ///
  /// # Errors
  ///
  /// As [`Self::get_reference_path`].
  pub fn get_path(&self, objects_root: &Path) -> XrfResult<PathBuf> {
    Self::get_reference_path(objects_root, &self.reference)
  }

  /// The path under `objects_root` of the object `reference` names, as the Level Editor resolves `$objects$`.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the reference, taken from a scene or a library, names no file below the root: one
  /// climbing out of it with `..`, carrying a drive, or empty.
  pub fn get_reference_path(objects_root: &Path, reference: &str) -> XrfResult<PathBuf> {
    let relative: HostRelativePath = HostRelativePath::new_file(reference, "Object reference")?;

    Ok(objects_root.join(relative.with_extension(Self::EXTENSION)))
  }
}

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
  fn is_held_at(&self, path: &Path) -> bool {
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

  /// Writes the scene's sidecar (`<level_path>.xrf.json`), then each object under `objects_root` after its own sidecar,
  /// then each library object and its thumbnail as copied (left alone where the file already holds them), then each
  /// texture under `textures_root` but a library object's own the root holds, then the scene: its main file at
  /// `level_path` and its parts in the directory beside it. Each sidecar goes before what it describes, so a write that
  /// fails leaves a stray sidecar, never a decompile taken for authored work.
  ///
  /// # Errors
  ///
  /// Returns an error, writing nothing, when [`EditableLevelScene::check_writable`] refuses the scene or a name taken
  /// from data names no file below its root; when a sidecar, object or texture cannot be written; or as
  /// [`EditableLevelScene::write_to_path`] does.
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
