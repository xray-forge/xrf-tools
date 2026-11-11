use std::collections::HashMap;
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::scene::level_scene_decompile::LevelSceneObjectFile;

/// The library objects a scene names: those held in memory (a decompile's own) by their reference, else read from
/// objects roots (`$objects$`) the first time each is named, from the first root holding it: a reference
/// `levels\labx8\labx8_sector_0` is `<root>/levels/labx8/labx8_sector_0.object`. A decompiled scene's own objects and
/// the library objects it names, such as GSC's detail objects, may stand in different roots.
pub(crate) struct LevelBuildLibrary<'a> {
  roots: Vec<PathBuf>,
  held: HashMap<&'a str, &'a EditableObjectFile>,
  objects: HashMap<String, EditableObjectFile>,
}

impl<'a> LevelBuildLibrary<'a> {
  pub(crate) fn new(roots: &[PathBuf], held: &'a [LevelSceneObjectFile]) -> Self {
    Self {
      roots: roots.to_vec(),
      held: held
        .iter()
        .map(|held| (held.reference.as_str(), &held.object))
        .collect(),
      objects: HashMap::new(),
    }
  }

  /// Reads every object of `references` the library neither holds nor has read yet.
  ///
  /// # Errors
  ///
  /// Returns an error naming the reference whose object no root holds or that does not read.
  pub(crate) fn load<'r, T: ByteOrder>(&mut self, references: impl IntoIterator<Item = &'r str>) -> XrfResult {
    for reference in references {
      if self.held.contains_key(reference) || self.objects.contains_key(reference) {
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

  /// The object of `reference`, held or once [`Self::load`] read it.
  pub(crate) fn get(&self, reference: &str) -> Option<&EditableObjectFile> {
    self
      .held
      .get(reference)
      .copied()
      .or_else(|| self.objects.get(reference))
  }
}
