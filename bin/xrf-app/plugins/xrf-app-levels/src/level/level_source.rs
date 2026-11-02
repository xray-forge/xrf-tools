use std::fmt::Display;
use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_material::XrayTextureScope;
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayRootProbe, XraySearch};

/// Where a compiled level is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelSource {
  /// A compiled level directory on disk, named by its filesystem path.
  Directory { path: String },
  /// A level of the mounted roots, named by its engine identity.
  Asset { logical_path: String },
}

impl LevelSource {
  /// The directory every installation keeps its levels under.
  pub const LEVELS_DIRECTORY: &'static str = "levels";

  /// The address as given, a path or an engine identity, for messages and the log.
  pub fn get_label(&self) -> &str {
    match self {
      Self::Directory { path } => path,
      Self::Asset { logical_path } => logical_path,
    }
  }

  /// Returns the level's filesystem path when its source provides one.
  pub fn get_physical_path(&self) -> Option<&Path> {
    match self {
      Self::Directory { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }

  /// The level's engine identity: the directory its own files are addressed under, for a directory the path its
  /// implied root gives it, which its files are read by, else `levels\<name>`.
  pub fn get_logical_directory(&self) -> Option<XrayLogicalPath> {
    match self {
      Self::Asset { logical_path } => XrayLogicalPath::new(logical_path).ok(),
      Self::Directory { path } => XrayRootProbe::implied_logical_path(path).or_else(|| {
        Path::new(path)
          .file_name()
          .and_then(|name| name.to_str())
          .and_then(|name| {
            XrayLogicalPath::new(Self::LEVELS_DIRECTORY)
              .and_then(|levels| levels.join(name))
              .ok()
          })
      }),
    }
  }

  /// Where its surfaces' textures and descriptors are read: beside the level, which the engine mounts as `$level$`,
  /// then the shared tree; the shared tree alone for a level with no engine identity.
  pub fn get_texture_scope(&self) -> XrayTextureScope {
    self
      .get_logical_directory()
      .map_or_else(XrayTextureScope::shared, XrayTextureScope::of_level)
  }

  /// The level's name, the last component of its engine identity, which the game graph knows it by.
  pub fn get_name(&self) -> Option<String> {
    self.get_logical_directory().map(|path| path.file_name().to_owned())
  }

  /// The level's name, for a read that finds the level's share of a game-wide file by it, as the spawn and its graph.
  ///
  /// # Errors
  ///
  /// Returns that the level has no engine identity to take a name from.
  pub fn require_name(&self) -> TauriResult<String> {
    self
      .get_name()
      .ok_or_else(|| format!("Level {} has no name the game knows it by", self.get_label()))
  }

  /// One of the level's files, from disk or out of the mounted roots.
  ///
  /// # Errors
  ///
  /// Returns why the file was refused or could not be read, or that the level has none.
  pub fn read_file(&self, search: &XraySearch, file: &str) -> TauriResult<Vec<u8>> {
    self
      .read_optional_file(search, file)?
      .ok_or_else(|| format!("Level '{}' has no '{file}'", self.get_label()))
  }

  /// One of the level's files through `search`, or `None` where the level has none: only a file that is not there, not
  /// one that cannot be read.
  ///
  /// # Errors
  ///
  /// Returns why the file was refused or could not be read.
  pub fn read_optional_file(&self, search: &XraySearch, file: &str) -> TauriResult<Option<Vec<u8>>> {
    let logical_path: XrayLogicalPath = self.to_file_logical_path(file)?;
    let found: Option<XrayAsset> = search
      .find(logical_path.as_str())
      .map_err(|error| format!("Rejected level file '{logical_path}': {error}"))?;

    found
      .map(|asset| search.read_asset(&asset))
      .transpose()
      .map_err(|error| format!("Failed to read level file '{logical_path}': {error}"))
  }

  /// A level file's engine identity, which the search reads it by: a level opened as a directory is read by the logical
  /// path its implied root gives it, which the open's roots, centred on it, search first, so its reads are recorded as
  /// an archived level's are.
  ///
  /// # Errors
  ///
  /// Returns why the file was refused: a name that would leave the level's directory (a root, a drive, `.` or `..`, as
  /// a spawn's motion names can be written), or a directory under no root a search would mount.
  pub fn to_file_logical_path(&self, file: &str) -> TauriResult<XrayLogicalPath> {
    let directory: XrayLogicalPath = match self {
      Self::Asset { logical_path } => {
        XrayLogicalPath::new(logical_path).map_err(|error| format!("Rejected level '{logical_path}': {error}"))?
      }
      Self::Directory { path } => XrayRootProbe::implied_logical_path(path).ok_or_else(|| {
        format!("Level directory '{path}' lies under no game data root, so no search of its roots reaches it")
      })?,
    };

    if file.is_empty() {
      return Err(format!("Rejected a level file of '{directory}' named by nothing"));
    }

    directory.join(file).map_err(|error| {
      format!("Rejected level file '{file}' of '{directory}': it names no file inside the level: {error}")
    })
  }

  /// Why one of its files, read, could not be taken for what it holds.
  pub fn describe_read_failure(&self, file: &str, error: &impl Display) -> String {
    format!("Failed to read '{file}' of level '{}': {error}", self.get_label())
  }
}
