use std::ffi::OsStr;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Component, Path};

use xrf_error::{XrfError, XrfResult};

/// One file name taken from data, spelled as authored, that cannot leave the directory it is joined to.
///
/// Both separators and the colon are refused on every host: Linux reads `..\outside` and `C:\outside` as one ordinary
/// component, which would carry the escape to the host that recognizes it.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HostFileName(String);

impl HostFileName {
  /// Validates `name` as one file name, naming it as `what` in the error.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the name is empty, `.`, `..`, or holds a separator or a colon.
  pub fn new(name: &str, what: &str) -> XrfResult<Self> {
    if is_host_file_name(name) {
      Ok(Self(name.to_string()))
    } else {
      Err(XrfError::new_invalid_error(format!(
        "{what} '{name}' must be a single file name, without a separator, a drive, or a traversal"
      )))
    }
  }

  /// The same name with `.extension` appended; the extension is a caller constant, never data.
  pub fn with_extension(&self, extension: &str) -> Self {
    let name: String = format!("{}.{extension}", self.0);

    debug_assert!(
      is_host_file_name(&name),
      "extension '{extension}' is not part of one file name"
    );

    Self(name)
  }

  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl AsRef<Path> for HostFileName {
  fn as_ref(&self) -> &Path {
    Path::new(&self.0)
  }
}

impl Display for HostFileName {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    formatter.write_str(&self.0)
  }
}

/// Whether `name` is one ordinary component on every host.
pub(crate) fn is_host_file_name(name: &str) -> bool {
  // With neither separator nor colon left, a name is one component or it is empty, `.`, or `..`.
  !name.contains(['\\', '/', ':'])
    && matches!(
      Path::new(name).components().next(),
      Some(Component::Normal(component)) if component == OsStr::new(name)
    )
}

#[cfg(test)]
mod tests {
  use super::HostFileName;

  #[test]
  fn keeps_a_file_name_as_authored() {
    for name in [
      "gamedata",
      "levels.part2",
      "weapons-v1",
      "ui_inGame2_PDA",
      "wpn_ak74 (old)",
      "...dds",
    ] {
      assert_eq!(HostFileName::new(name, "Name").expect("one file name").as_str(), name);
    }
  }

  #[test]
  fn rejects_a_name_that_would_leave_its_destination() {
    // A rooted or drive-qualified name does not escape the destination so much as discard it: `join` returns it
    // whole. The Windows spellings are refused on Linux too, where they would otherwise write innocent file names
    // and escape on the host that reads them back.
    for name in [
      "",
      ".",
      "..",
      "..\\outside",
      "../outside",
      "nested\\name",
      "nested/name",
      "\\rooted",
      "/rooted",
      "C:\\outside",
      "C:outside",
      "name:stream",
    ] {
      assert!(
        HostFileName::new(name, "Name").is_err(),
        "'{name}' must not name a file"
      );
    }
  }

  #[test]
  fn appends_an_extension() {
    assert_eq!(
      HostFileName::new("wpn_ak74", "Name")
        .expect("one file name")
        .with_extension("dds")
        .as_str(),
      "wpn_ak74.dds"
    );
  }
}
