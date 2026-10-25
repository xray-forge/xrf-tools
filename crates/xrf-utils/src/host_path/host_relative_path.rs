use std::ffi::OsStr;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};

use crate::host_path::host_file_name::is_host_file_name;
use crate::host_path::{HostFileName, format_path};

/// A path taken from data, spelled as authored, that cannot leave the root it is joined to; empty names the root.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HostRelativePath(PathBuf);

impl HostRelativePath {
  /// Crosses a name spelled with either separator into host components, dropping empty ones.
  ///
  /// # Errors
  ///
  /// Returns an invalid error naming the first component that is `.`, `..`, or holds a colon.
  pub fn new(name: &str, what: &str) -> XrfResult<Self> {
    let mut path: PathBuf = PathBuf::new();

    for component in name.split(['\\', '/']).filter(|part| !part.is_empty()) {
      if !is_host_file_name(component) {
        return Err(XrfError::new_invalid_error(format!(
          "{what} '{name}' contains '{component}', which would leave its destination"
        )));
      }

      path.push(component);
    }

    Ok(Self(path))
  }

  /// This path with one more file name below it.
  pub fn join(&self, name: &HostFileName) -> Self {
    Self(self.0.join(name))
  }

  /// This path with `other` below it.
  pub fn join_path(&self, other: &HostRelativePath) -> Self {
    Self(self.0.join(&other.0))
  }

  /// The same path with its last extension replaced or set; the extension is a caller constant, never data.
  pub fn with_extension(&self, extension: &str) -> Self {
    debug_assert!(
      is_host_file_name(&format!("name.{extension}")),
      "extension '{extension}' is not part of one file name"
    );

    Self(self.0.with_extension(extension))
  }

  /// The directory holding this path, or `None` for the root itself.
  pub fn parent(&self) -> Option<Self> {
    self.0.parent().map(|parent| Self(parent.to_path_buf()))
  }

  /// The last component, or `None` for the root itself.
  pub fn file_name(&self) -> Option<&OsStr> {
    self.0.file_name()
  }

  pub fn is_empty(&self) -> bool {
    self.0.as_os_str().is_empty()
  }

  pub fn as_path(&self) -> &Path {
    &self.0
  }
}

impl From<HostFileName> for HostRelativePath {
  fn from(name: HostFileName) -> Self {
    Self(PathBuf::from(name.as_str()))
  }
}

impl AsRef<Path> for HostRelativePath {
  fn as_ref(&self) -> &Path {
    &self.0
  }
}

impl Display for HostRelativePath {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    Display::fmt(&format_path(&self.0), formatter)
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use super::HostRelativePath;
  use crate::HostFileName;

  fn relative(name: &str) -> PathBuf {
    HostRelativePath::new(name, "Name")
      .expect("a safe relative path")
      .as_path()
      .to_path_buf()
  }

  #[test]
  fn crosses_either_separator_into_host_components() {
    let expected: PathBuf = Path::new("configs").join("system.ltx");

    assert_eq!(relative("configs\\system.ltx"), expected);
    assert_eq!(relative("configs/system.ltx"), expected);
    assert_eq!(relative("system.ltx"), Path::new("system.ltx"));
  }

  #[test]
  fn keeps_case_and_drops_empty_components() {
    assert_eq!(relative("Gamedata\\"), Path::new("Gamedata"));
    assert_eq!(relative("\\ui\\\\Sheet"), Path::new("ui").join("Sheet"));
    assert_eq!(relative(""), PathBuf::new());
  }

  #[test]
  fn refuses_a_component_that_would_leave_its_destination() {
    for name in [
      "..",
      "..\\outside",
      "../outside",
      "configs\\..\\..\\outside",
      "configs/./system.ltx",
      "C:\\outside",
      "C:outside",
      "configs\\system.ltx:stream",
    ] {
      assert!(
        HostRelativePath::new(name, "Name").is_err(),
        "'{name}' must not become a host path"
      );
    }
  }

  #[test]
  fn joins_a_file_name_below_itself() {
    let sheet: HostRelativePath = HostRelativePath::new("ui\\ui_icons", "Name").expect("a safe relative path");
    let sprite: HostFileName = HostFileName::new("ui_icon_PDA", "Name").expect("one file name");

    assert_eq!(
      sheet.join(&sprite.with_extension("dds")).as_path(),
      Path::new("ui").join("ui_icons").join("ui_icon_PDA.dds")
    );
  }
}
