use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::path::XrayLogicalPath;
use crate::{FsgameFile, XrayMountMode, XraySearchPlan};

/// An SDK installation: the editors' directory and the `fs.ltx` there, which lays out the game's tree and beside it the
/// SDK's own authoring tree, rawdata, under `$server_data_root$`.
///
/// Rawdata is searched apart from gamedata: both hold `textures\`, `sounds\` and `levels\`, so one search over both
/// would answer a rawdata path with a game file. [`Self::to_rawdata_search_plan`] mounts rawdata as a root of its own,
/// and [`Self::get_rawdata_prefix`] turns an editor alias such as `$objects$` into the logical path it searches under.
#[derive(Clone, Debug)]
pub struct XraySdkInstallation {
  file: FsgameFile,
}

impl XraySdkInstallation {
  /// The file the SDK's editors read their layout from, beside the `fsgame.ltx` the game they launch reads.
  pub const FILE_NAME: &'static str = "fs.ltx";

  /// The alias the SDK's `fs.ltx` declares rawdata under.
  pub const RAWDATA_ALIAS: &'static str = "$server_data_root$";

  /// The label a rawdata search step carries.
  pub const RAWDATA_LABEL: &'static str = "rawdata";

  /// The SDK layout file a directory holds, if it holds one.
  pub fn find(directory: impl AsRef<Path>) -> Option<PathBuf> {
    Some(directory.as_ref().join(Self::FILE_NAME)).filter(|path| path.is_file())
  }

  /// Reads the SDK layout of an editors' directory.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory holds no `fs.ltx`, or it cannot be read, decoded, or parsed.
  pub fn read(directory: impl AsRef<Path>) -> XrfResult<Self> {
    let directory: &Path = directory.as_ref();
    let path: PathBuf = Self::find(directory).ok_or_else(|| {
      XrfError::new_read_error(format!(
        "{} holds no {}, the layout the SDK's editors read",
        format_path(directory),
        Self::FILE_NAME
      ))
    })?;

    Ok(Self::from(FsgameFile::read_file(path)?))
  }

  /// The parsed layout file, for any alias this type does not name.
  pub fn get_file(&self) -> &FsgameFile {
    &self.file
  }

  /// The rawdata directory, or `None` when the layout declares none it can resolve.
  pub fn get_rawdata_root(&self) -> Option<PathBuf> {
    self.file.resolve(Self::RAWDATA_ALIAS)
  }

  /// The logical path an alias's directory has inside rawdata, `objects` for `$objects$` and the empty path for rawdata
  /// itself, or `None` when the alias is undeclared, unresolved, or lies outside rawdata as `$game_data$` does.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory's path inside rawdata is not Unicode, or not a valid logical path.
  pub fn get_rawdata_prefix(&self, alias: &str) -> XrfResult<Option<String>> {
    let Some(rawdata) = self.get_rawdata_root() else {
      return Ok(None);
    };
    let Some(directory) = self.file.resolve(alias) else {
      return Ok(None);
    };
    let Ok(inside) = directory.strip_prefix(&rawdata) else {
      return Ok(None);
    };

    if inside.as_os_str().is_empty() {
      return Ok(Some(String::new()));
    }

    let segments: Vec<&str> = inside
      .components()
      .map(|component| {
        component.as_os_str().to_str().ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "{alias} lies under rawdata at {}, a path that is not Unicode",
            format_path(&directory)
          ))
        })
      })
      .collect::<XrfResult<_>>()?;

    XrayLogicalPath::normalize(&segments.join("\\")).map(Some)
  }

  /// A search over rawdata alone, mounted as one loose root: a logical path in it is a path under rawdata,
  /// `objects\editor\axis.object` for the object `$objects$` holds as `editor\axis.object`.
  ///
  /// # Errors
  ///
  /// Returns an error when the layout declares no rawdata it can resolve.
  pub fn to_rawdata_search_plan(&self) -> XrfResult<XraySearchPlan> {
    let rawdata: PathBuf = self.get_rawdata_root().ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "SDK layout at {} declares no {} it can resolve",
        format_path(self.file.get_root()),
        Self::RAWDATA_ALIAS
      ))
    })?;

    XraySearchPlan::new().with_root_mode(Self::RAWDATA_LABEL, rawdata, XrayMountMode::Directory)
  }
}

impl From<FsgameFile> for XraySdkInstallation {
  fn from(file: FsgameFile) -> Self {
    Self { file }
  }
}
