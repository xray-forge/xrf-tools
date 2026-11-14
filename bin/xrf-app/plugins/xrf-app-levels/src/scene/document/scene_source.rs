use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_editable_level::EditableLevelFile;
use xrf_vfs::{XrayMountMode, XrayRoots};

/// Where a Level Editor scene is read from: its `.level` on disk. The scene is searched for from the directory holding
/// the `.level`, its parts in the directory beside it named after it, so a scene opens wherever it lies.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneSource {
  /// The `.level` file's filesystem path.
  pub path: String,
}

impl SceneSource {
  /// The directory a rawdata tree keeps library objects in, beside the `levels` directory holding its scenes.
  pub const OBJECTS_DIRECTORY: &'static str = "objects";

  /// The scene's name, the `.level`'s without its extension, which names its part directory.
  ///
  /// # Errors
  ///
  /// Returns an error for a path naming no `.level` file.
  pub fn require_name(&self) -> TauriResult<String> {
    let path: &Path = Path::new(&self.path);
    let is_level: bool = path
      .extension()
      .and_then(OsStr::to_str)
      .is_some_and(|extension| extension.eq_ignore_ascii_case(EditableLevelFile::EXTENSION));

    path
      .file_stem()
      .and_then(OsStr::to_str)
      .filter(|name| is_level && !name.is_empty())
      .map(str::to_owned)
      .ok_or_else(|| format!("Scene '{}' names no .{} file", self.path, EditableLevelFile::EXTENSION))
  }

  /// The roots the scene is searched in: the directory holding its `.level`, mounted as a loose tree whatever lies
  /// beside it, so its files are read by their paths below it and a commit to any of them is followed.
  ///
  /// # Errors
  ///
  /// Returns an error for a path that lies in no directory.
  pub fn get_roots(&self) -> TauriResult<XrayRoots> {
    let directory: &Path = Path::new(&self.path)
      .parent()
      .filter(|directory| !directory.as_os_str().is_empty())
      .ok_or_else(|| format!("Scene '{}' lies in no directory", self.path))?;

    Ok(XrayRoots::from_paths(
      [directory.to_path_buf()],
      XrayMountMode::Directory,
    ))
  }

  /// Where the scene's library objects are looked for: the directories `given`, in search order, else `objects` beside
  /// the directory holding the `.level`, as a rawdata tree keeps `levels` and `objects` side by side.
  pub fn get_objects_roots(&self, given: Option<Vec<String>>) -> XrayRoots {
    let paths: Vec<PathBuf> = match given {
      Some(given) => given.into_iter().map(PathBuf::from).collect(),
      None => Path::new(&self.path)
        .parent()
        .and_then(Path::parent)
        .map(|rawdata| rawdata.join(Self::OBJECTS_DIRECTORY))
        .into_iter()
        .collect(),
    };

    XrayRoots::from_paths(paths, XrayMountMode::Directory)
  }
}
