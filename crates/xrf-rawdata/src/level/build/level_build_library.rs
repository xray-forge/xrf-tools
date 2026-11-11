use std::collections::HashMap;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::scene::level_scene_decompile::LevelSceneObjectFile;

/// The library objects a scene names, read from objects roots (`$objects$`) the first time each is named, from the first
/// root holding it: a reference `levels\labx8\labx8_sector_0` is `<root>/levels/labx8/labx8_sector_0.object`. A
/// decompiled scene's own objects and the library objects it names, such as GSC's detail objects, may stand in
/// different roots.
pub(crate) struct LevelBuildLibrary {
  roots: Vec<PathBuf>,
  objects: HashMap<String, EditableObjectFile>,
}

impl LevelBuildLibrary {
  pub(crate) fn new(roots: &[PathBuf]) -> Self {
    Self {
      roots: roots.to_vec(),
      objects: HashMap::new(),
    }
  }

  /// Reads every object of `references` the library has not read yet.
  ///
  /// # Errors
  ///
  /// Returns an error naming the reference whose object no root holds or that does not read.
  pub(crate) fn load<'a, T: ByteOrder>(&mut self, references: impl IntoIterator<Item = &'a str>) -> XrfResult {
    for reference in references {
      if self.objects.contains_key(reference) {
        continue;
      }

      let candidates: Vec<PathBuf> = self
        .roots
        .iter()
        .map(|root| LevelSceneObjectFile::get_reference_path(root, reference))
        .collect();
      let path: &Path = candidates
        .iter()
        .find(|path| path.is_file())
        .or(candidates.first())
        .ok_or_else(|| XrfError::new_invalid_error("A build names no objects root"))?;
      let object: EditableObjectFile = EditableObjectFile::read_from_path::<T, _>(&path).map_err(|error| {
        XrfError::new_read_error(format!(
          "Library object '{reference}' was not read from {}: {error}",
          format_path(&path)
        ))
      })?;

      self.objects.insert(reference.to_owned(), object);
    }

    Ok(())
  }

  /// The object of `reference`, once [`Self::load`] read it.
  pub(crate) fn get(&self, reference: &str) -> Option<&EditableObjectFile> {
    self.objects.get(reference)
  }
}
