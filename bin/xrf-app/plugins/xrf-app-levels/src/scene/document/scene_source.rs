use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_editable_level::EditableLevelFile;
use xrf_vfs::{XrayMountMode, XrayRoots};

/// A Level Editor scene's `.level` on disk. The scene is searched from the directory holding it, its parts in the
/// directory beside it named after it, so a scene opens wherever it lies.
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

  /// The directory an SDK keeps the game's data in, beside its `rawdata`.
  pub const GAME_DATA_DIRECTORY: &'static str = "gamedata";

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

  /// The directory holding the `.level`, mounted as a loose tree whatever lies beside it, so the scene's files are read
  /// by their paths below it and a commit to any of them is followed.
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
  /// the directory holding the `.level`, as in a rawdata tree.
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

  /// Where the scene's textures and shaders are searched for: the roots `given`, else the SDK's `gamedata` beside the
  /// `rawdata` holding the scene's `levels` where it exists, else none.
  pub fn get_game_roots(&self, given: Option<XrayRoots>) -> XrayRoots {
    given.unwrap_or_else(|| {
      XrayRoots::from_paths(
        Path::new(&self.path)
          .parent()
          .and_then(Path::parent)
          .and_then(Path::parent)
          .map(|beside| beside.join(Self::GAME_DATA_DIRECTORY))
          .filter(|game_data| game_data.is_dir()),
        XrayMountMode::Directory,
      )
    })
  }
}
