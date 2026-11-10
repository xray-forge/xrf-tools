use std::collections::HashMap;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::scene::level_scene_decompile::LevelSceneObjectFile;

/// The library objects a scene places, read from an objects root (`$objects$`) the first time each is named: a
/// reference `levels\labx8\labx8_sector_0` is `<root>/levels/labx8/labx8_sector_0.object`.
pub(crate) struct LevelBuildLibrary {
  root: PathBuf,
  objects: HashMap<String, EditableObjectFile>,
}

impl LevelBuildLibrary {
  pub(crate) fn new(root: &Path) -> Self {
    Self {
      root: root.to_path_buf(),
      objects: HashMap::new(),
    }
  }

  /// Reads every object of `references` the library has not read yet.
  ///
  /// # Errors
  ///
  /// Returns an error naming the reference whose object is missing or does not read.
  pub(crate) fn load<'a, T: ByteOrder>(&mut self, references: impl IntoIterator<Item = &'a str>) -> XrfResult {
    for reference in references {
      if self.objects.contains_key(reference) {
        continue;
      }

      let path: PathBuf = LevelSceneObjectFile::get_reference_path(&self.root, reference);
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
