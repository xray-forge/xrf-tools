use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_level::EditableLevelScene;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

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
  pub fn get_path(&self, objects_root: &Path) -> PathBuf {
    Self::get_reference_path(objects_root, &self.reference)
  }

  /// The path under `objects_root` of the object `reference` names, as the Level Editor resolves `$objects$`.
  pub fn get_reference_path(objects_root: &Path, reference: &str) -> PathBuf {
    let mut path: PathBuf = objects_root.join(reference.split('\\').collect::<PathBuf>());

    path.set_extension(Self::EXTENSION);
    path
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
  pub fn get_paths(&self, objects_root: &Path) -> (PathBuf, PathBuf) {
    let path: PathBuf = LevelSceneObjectFile::get_reference_path(objects_root, &self.reference);
    let thumbnail: PathBuf = path.with_extension(Self::THUMBNAIL_EXTENSION);

    (path, thumbnail)
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
}

impl LevelSceneTextureFile {
  /// The extension the name leaves off.
  pub const EXTENSION: &'static str = "dds";

  /// Its path under `textures_root`, the name's folders as directories.
  pub fn get_path(&self, textures_root: &Path) -> PathBuf {
    Self::get_named_path(textures_root, &self.name)
  }

  /// The path under `textures_root` of the texture `name` names, as the editor resolves `$game_textures$`.
  pub fn get_named_path(textures_root: &Path, name: &str) -> PathBuf {
    let mut path: PathBuf = textures_root.join(name.split('\\').collect::<PathBuf>());

    path.set_extension(Self::EXTENSION);
    path
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
  /// What a write would replace of the textures the objects need, under `textures_root`.
  pub fn get_existing_texture_paths(&self, textures_root: &Path) -> Vec<PathBuf> {
    self
      .textures
      .iter()
      .map(|texture| texture.get_path(textures_root))
      .filter(|path| path.exists())
      .collect()
  }

  /// What a write would replace of the objects the scene places, under `objects_root`, and their sidecars; a library
  /// copy that would write a file's own bytes again replaces nothing, as where the objects root is the library's.
  pub fn get_existing_object_paths(&self, objects_root: &Path) -> Vec<PathBuf> {
    let library = self.library.iter().flat_map(|file| {
      let (path, thumbnail) = file.get_paths(objects_root);

      [
        (path, Some(file.object.as_slice())),
        (thumbnail, file.thumbnail.as_deref()),
      ]
    });

    self
      .objects
      .iter()
      .flat_map(|object| {
        let path: PathBuf = object.get_path(objects_root);

        [(ObjectProvenance::sidecar_path(&path), None), (path, None)]
      })
      .chain(library)
      .filter(|(path, bytes)| path.exists() && !bytes.is_some_and(|bytes| Self::is_same_file(path, bytes)))
      .map(|(path, _)| path)
      .collect()
  }

  /// Whether the file at `path` holds `bytes`.
  fn is_same_file(path: &Path, bytes: &[u8]) -> bool {
    std::fs::read(path).is_ok_and(|held| held == bytes)
  }

  /// What a write to `level_path` would replace, of the scene's main file, its part directory and its sidecar: what a
  /// front-end asks about before writing over an earlier decompile or an authored scene.
  pub fn get_existing_paths(level_path: &Path) -> Vec<PathBuf> {
    [
      level_path.to_path_buf(),
      EditableLevelScene::get_parts_directory(level_path),
      LevelSceneProvenance::sidecar_path(level_path),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
  }

  /// Writes the scene's sidecar (`<level_path>.xrf.json`), then each object under `objects_root` after its own sidecar,
  /// then each library object and its thumbnail as copied (left alone where the file already holds them), then each
  /// texture under `textures_root`, then the scene: its main file at `level_path` and its parts in the
  /// directory beside it. Each sidecar goes before what it describes, so a write that fails leaves a stray sidecar,
  /// never a decompile taken for authored work.
  ///
  /// # Errors
  ///
  /// Returns an error, writing nothing, when [`EditableLevelScene::check_writable`] refuses the scene; when a sidecar,
  /// object or texture cannot be written; or as [`EditableLevelScene::write_to_path`] does.
  pub fn write_to_path<T: ByteOrder>(&self, level_path: &Path, objects_root: &Path, textures_root: &Path) -> XrfResult {
    // Refused before the sidecar is written, so a sidecar never lands beside an authored scene left as it was.
    self.scene.check_writable(level_path)?;

    Self::write_file(
      &LevelSceneProvenance::sidecar_path(level_path),
      self.provenance.to_json()?.as_bytes(),
    )?;

    for object in &self.objects {
      let path: PathBuf = object.get_path(objects_root);

      Self::write_file(
        &ObjectProvenance::sidecar_path(&path),
        object.provenance.to_json()?.as_bytes(),
      )?;
      Self::write_file(&path, &object.object.write_to_bytes::<T>()?)?;
    }

    for file in &self.library {
      let (path, thumbnail) = file.get_paths(objects_root);

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

    for texture in &self.textures {
      Self::write_file(&texture.get_path(textures_root), &texture.bytes)?;
    }

    self.scene.write_to_path::<T, _>(&level_path)
  }

  fn write_file(path: &Path, bytes: &[u8]) -> XrfResult {
    StagedFile::write_creating_directories(path, bytes)
      .map_err(|error| XrfError::new_io_error(format!("{} was not written: {error}", format_path(path)), error.kind()))
  }
}
