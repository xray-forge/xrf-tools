use std::path::{Path, PathBuf};

use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_utils::HostRelativePath;

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
