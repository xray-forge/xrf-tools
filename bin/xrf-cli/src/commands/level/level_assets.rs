//! Where one compiled level's files come from, so a command reads the same level loose or archived.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelFile, LevelFileSource};
use xrf_utils::format_path;
use xrf_vfs::XraySearch;

/// The directory every installation keeps its levels under.
const LEVELS_DIRECTORY: &str = "levels";

/// One compiled level's files, addressed however the level is stored.
pub enum LevelAssets {
  /// A level directory on the host filesystem, read by joining paths.
  Directory(PathBuf),
  /// A level of mounted roots, read by its engine identity, `levels\<name>\<file>`.
  Asset { search: XraySearch, name: String },
}

impl LevelAssets {
  /// Takes a level directory as it sits on disk.
  pub fn open_directory(path: &Path) -> Self {
    Self::Directory(path.to_path_buf())
  }

  /// Opens a level of mounted roots by the name the installation knows it by.
  ///
  /// # Errors
  ///
  /// Returns an error when the level holds no `level` file, which is what tells a name that exists from one that
  /// does not - a directory of only lightmaps is not a level.
  pub fn open_asset(search: XraySearch, name: &str) -> XrfResult<Self> {
    let assets: Self = Self::Asset {
      name: name.to_owned(),
      search,
    };
    let bundle: String = assets.describe_file(LevelFile::FILE_NAME);

    // Found, not read: an archived bundle is the largest file a level ships, and its reader reads it next.
    if let Self::Asset { search, .. } = &assets
      && search.find(&bundle)?.is_none()
    {
      return Err(XrfError::new_not_found_error(format!(
        "Level '{name}' was not found in the mounted roots, which hold no '{bundle}' for it"
      )));
    }

    Ok(assets)
  }

  /// Reads one of the level's files, or `None` when the level ships without it.
  ///
  /// # Errors
  ///
  /// Returns an error when the file is there and cannot be read.
  pub fn read(&self, file: &str) -> XrfResult<Option<Vec<u8>>> {
    match self {
      Self::Directory(path) => {
        let path: PathBuf = path.join(file);

        if !path.is_file() {
          return Ok(None);
        }

        Ok(Some(fs::read(&path).map_err(|error| {
          XrfError::new_read_error(format!(
            "Level file was not read: {}, error: {error}",
            format_path(&path)
          ))
        })?))
      }
      Self::Asset { search, .. } => match search.find(&self.describe_file(file))? {
        Some(asset) => Ok(Some(search.read_asset(&asset)?)),
        None => Ok(None),
      },
    }
  }

  /// The level as a subject a report names.
  pub fn describe(&self) -> String {
    match self {
      Self::Directory(path) => format_path(path).to_string(),
      Self::Asset { name, .. } => format!("{LEVELS_DIRECTORY}\\{name}"),
    }
  }

  /// One of the level's files, spelled the way the subject is.
  pub fn describe_file(&self, file: &str) -> String {
    match self {
      Self::Directory(path) => format_path(&path.join(file)).to_string(),
      Self::Asset { name, .. } => format!("{LEVELS_DIRECTORY}\\{name}\\{file}"),
    }
  }

  /// Every level name the mounted roots hold, which is what a caller offers when one was not named.
  ///
  /// # Errors
  ///
  /// Returns an error when the levels directory cannot be listed.
  pub fn list_names(search: &XraySearch) -> XrfResult<Vec<String>> {
    let mut names: Vec<String> = Vec::new();

    for child in search.list_children(LEVELS_DIRECTORY)?.directories {
      if search
        .find(&format!("{LEVELS_DIRECTORY}\\{child}\\{}", LevelFile::FILE_NAME))?
        .is_some()
      {
        names.push(child);
      }
    }

    names.sort();

    Ok(names)
  }
}

impl LevelFileSource for LevelAssets {
  fn read(&self, name: &str) -> XrfResult<Option<Vec<u8>>> {
    LevelAssets::read(self, name)
  }

  fn describe(&self, name: &str) -> String {
    self.describe_file(name)
  }
}
