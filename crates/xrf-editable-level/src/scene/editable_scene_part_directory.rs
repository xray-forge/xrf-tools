use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::scene::editable_scene_part_source::EditableScenePartSource;

/// A scene's part directory on the host, the `.level` path without its extension.
pub(crate) struct EditableScenePartDirectory {
  pub(crate) directory: PathBuf,
}

impl EditableScenePartSource for EditableScenePartDirectory {
  fn list_files(&self) -> XrfResult<Vec<String>> {
    if !self.directory.is_dir() {
      return Ok(Vec::new());
    }

    let mut names: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&self.directory)? {
      let entry = entry?;

      if entry.file_type()?.is_file()
        && let Some(name) = entry.file_name().to_str()
      {
        names.push(name.to_owned());
      }
    }

    Ok(names)
  }

  fn read_file(&self, name: &str) -> XrfResult<Vec<u8>> {
    let path: PathBuf = self.directory.join(name);

    std::fs::read(&path).map_err(|error| {
      XrfError::new_io_error(
        format!("Level part {} was not read: {error}", format_path(&path)),
        error.kind(),
      )
    })
  }
}
