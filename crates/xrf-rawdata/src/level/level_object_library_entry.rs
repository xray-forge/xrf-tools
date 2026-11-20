use std::path::{Path, PathBuf};

/// One `.object` file under an object library root, and the reference a scene object names it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelObjectLibraryEntry {
  pub path: PathBuf,
  /// Its path under the root without the extension, folders joined by `\`, as `$objects$` references name it.
  pub reference: String,
}

impl LevelObjectLibraryEntry {
  /// The entry of the file at `path` under `root`.
  pub(crate) fn new(root: &Path, path: PathBuf) -> Self {
    let reference: String = path
      .strip_prefix(root)
      .unwrap_or(&path)
      .with_extension("")
      .components()
      .map(|component| component.as_os_str().to_string_lossy().into_owned())
      .collect::<Vec<String>>()
      .join("\\");

    Self { path, reference }
  }

  /// Whether it stands in the folder `directory` of its root, or one below it, any case.
  pub(crate) fn is_under(&self, directory: &str) -> bool {
    self
      .reference
      .split_once('\\')
      .is_some_and(|(first, _)| first.eq_ignore_ascii_case(directory))
  }
}
