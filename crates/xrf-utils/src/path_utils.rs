use std::fmt::{Display, Formatter, Result as FmtResult};
use std::path::{Component, Path, PathBuf};

use xrf_error::XrfResult;

struct HostPath<'a> {
  path: Option<&'a Path>,
  fallback: &'a str,
}

impl Display for HostPath<'_> {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    match self.path {
      Some(path) => Display::fmt(&path.display(), formatter),
      None => formatter.write_str(self.fallback),
    }
  }
}

/// Render a host path the way it is shown to a person: platform-native separators, and the lossy
/// substitution `Path::display` already performs for a name that is not valid Unicode.
///
/// This is the one boundary for human-facing host path output, so a presentation change happens here
/// rather than at every call site. It answers a different question than `to_portable_path_string`,
/// which is a deterministic wire or report key, and than `XrayLogicalPath`, which is an engine
/// identity. Rendering, never a write address: resolve a path to write through the VFS instead.
///
/// The returned type is deliberately opaque, so the rendering can change without breaking signatures,
/// and it borrows rather than allocating, which is what a format argument needs.
#[inline]
pub fn format_path<P: AsRef<Path> + ?Sized>(path: &P) -> impl Display + '_ {
  HostPath {
    path: Some(path.as_ref()),
    fallback: "",
  }
}

/// Render a host path that may be absent, naming the absent case explicitly.
///
/// The fallback is the caller's word, because what a missing path means is the caller's domain: an
/// `Ltx` built in memory is `virtual`, and nothing here should have to know that.
#[inline]
pub fn format_path_or<'a, P: AsRef<Path> + ?Sized>(path: Option<&'a P>, fallback: &'a str) -> impl Display + 'a {
  HostPath {
    path: path.map(AsRef::as_ref),
    fallback,
  }
}

/// Render a host path with forward slashes, for a wire contract or a report key.
///
/// Not to be confused with `XrayLogicalPath::normalize`, which answers the other domain: that one
/// produces an engine identity (`\`-separated, lower case), this one renders a host path so the same
/// project reports the same key on Windows and Linux. Four crates had grown their own copy of these
/// three tokens before this existed.
#[inline]
pub fn to_portable_path_string<T: AsRef<Path>>(path: T) -> String {
  path.as_ref().to_string_lossy().replace('\\', "/")
}

/// Resolve a host path to an absolute one, against the current directory and without consulting the filesystem.
///
/// Lexical on purpose. `canonicalize` refuses a path that does not exist yet, which a destination typically does not,
/// and on Windows it answers the verbatim `\\?\` form that no person typed and few tools accept. Folding `.` and `..`
/// by component gives the one spelling a person can act on for a path that may still be about to be created.
///
/// A symlink is therefore not followed, so the result names where the path says it goes rather than where it lands.
/// Canonicalize instead where that difference matters.
///
/// # Errors
///
/// Returns an IO error when the path is relative and the current directory cannot be read.
pub fn to_absolute_path<T: AsRef<Path>>(path: T) -> XrfResult<PathBuf> {
  let path: &Path = path.as_ref();
  let absolute: PathBuf = if path.is_absolute() {
    path.to_path_buf()
  } else {
    std::env::current_dir()?.join(path)
  };
  let mut normalized: PathBuf = PathBuf::new();

  for component in absolute.components() {
    match component {
      Component::CurDir => {}
      // A root has no parent, so `pop` leaves it alone rather than climbing past it.
      Component::ParentDir => {
        normalized.pop();
      }
      _ => normalized.push(component.as_os_str()),
    }
  }

  Ok(normalized)
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use super::{format_path, format_path_or, to_absolute_path, to_portable_path_string};

  #[test]
  fn renders_separators_the_same_way_on_every_platform() {
    assert_eq!(
      to_portable_path_string(PathBuf::from("configs").join("gameplay").join("dialogs.xml")),
      "configs/gameplay/dialogs.xml"
    );
    assert_eq!(to_portable_path_string("already/portable"), "already/portable");
    assert_eq!(to_portable_path_string(""), "");
  }

  #[test]
  fn renders_a_host_path_with_native_separators() {
    let path: PathBuf = PathBuf::from("configs").join("gameplay").join("dialogs.xml");

    assert_eq!(format_path(&path).to_string(), path.display().to_string());
    assert_eq!(format_path(Path::new("system.ltx")).to_string(), "system.ltx");
    assert_eq!(format_path("system.ltx").to_string(), "system.ltx");
  }

  #[test]
  fn renders_an_empty_path_as_nothing() {
    assert_eq!(format_path("").to_string(), "");
  }

  #[test]
  fn keeps_a_backslash_that_is_part_of_the_name() {
    // Only Windows reads this as a separator; on Unix it is an ordinary character and must survive.
    assert_eq!(format_path("configs\\system.ltx").to_string(), "configs\\system.ltx");
  }

  #[test]
  fn names_the_absent_path_with_the_supplied_word() {
    assert_eq!(
      format_path_or(Some(Path::new("system.ltx")), "virtual").to_string(),
      "system.ltx"
    );
    assert_eq!(format_path_or(None::<&Path>, "virtual").to_string(), "virtual");
    assert_eq!(format_path_or(None::<&Path>, "").to_string(), "");
  }

  #[test]
  fn resolves_a_relative_path_against_the_current_directory() {
    let current: PathBuf = std::env::current_dir().expect("a current directory");

    assert_eq!(to_absolute_path("packed").expect("resolves"), current.join("packed"));
    assert_eq!(to_absolute_path("").expect("resolves"), current);
  }

  #[test]
  fn folds_the_components_that_only_name_a_place_indirectly() {
    let current: PathBuf = std::env::current_dir().expect("a current directory");

    assert_eq!(
      to_absolute_path(Path::new(".").join("out").join("..").join("packed")).expect("resolves"),
      current.join("packed")
    );
  }

  #[test]
  fn keeps_an_absolute_path_absolute_without_touching_the_filesystem() {
    let root: PathBuf = std::env::current_dir()
      .expect("a current directory")
      .ancestors()
      .last()
      .expect("a root")
      .to_path_buf();
    let absent: PathBuf = root.join("no_such_directory").join("gamedata");

    assert!(!absent.exists(), "the case worth covering is a path that is not there");
    assert_eq!(to_absolute_path(&absent).expect("resolves"), absent);
  }

  #[test]
  fn never_climbs_past_the_root() {
    let root: PathBuf = std::env::current_dir()
      .expect("a current directory")
      .ancestors()
      .last()
      .expect("a root")
      .to_path_buf();

    assert_eq!(to_absolute_path(root.join("..").join("..")).expect("resolves"), root);
  }

  /// A Unix filename is bytes, not text, so it can be a valid path and still not be valid Unicode.
  /// `to_str` returns `None` for it, which is what used to be unwrapped; rendering must not.
  #[test]
  #[cfg(unix)]
  fn renders_a_path_that_is_not_valid_unicode() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path: PathBuf = PathBuf::from(OsStr::from_bytes(b"configs/\xff_broken.ltx"));

    assert!(path.to_str().is_none());
    assert_eq!(format_path(&path).to_string(), "configs/\u{fffd}_broken.ltx");
    assert_eq!(
      format_path_or(Some(&path), "virtual").to_string(),
      "configs/\u{fffd}_broken.ltx"
    );
  }
}
