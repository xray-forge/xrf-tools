//! Names and the host paths they cross into: the only place this crate decides separators.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// Converts an engine name into host path components.
///
/// Built from components so the platform inserts its own separator: pushed whole, `configs\system.ltx` is a single
/// component to `std::path` on Linux, which unpacks a tree as flat files with backslashes in their names.
///
/// Empty components are dropped, because a volume's entry point carries a trailing separator (`gamedata\`).
pub(crate) fn to_host_relative(name: &str) -> XrfResult<PathBuf> {
  let mut path: PathBuf = PathBuf::new();

  for component in name.split(['\\', '/']).filter(|part| !part.is_empty()) {
    if matches!(component, "." | "..") || component.contains(':') {
      return Err(XrfError::new_invalid_error(format!(
        "Archive entry path '{name}' contains unsafe component '{component}'"
      )));
    }

    path.push(component);
  }

  Ok(path)
}

/// Express a host path as the engine name relative to the root it was found under.
///
/// The counterpart of [`to_host_relative`], and the direction packing crosses in.
///
/// # Errors
///
/// Returns an error when the path is not below the source root, which would mean the walk left it, and when the host
/// path is not valid Unicode, which leaves no name to write: the archive has to say so rather than drop the file.
pub(crate) fn to_archive_name(source: &Path, path: &Path) -> XrfResult<String> {
  let relative: &Path = path.strip_prefix(source).map_err(|_| {
    XrfError::new_unexpected_error(format!(
      "Packing source '{}' is not below its root '{}'",
      format_path(path),
      format_path(source)
    ))
  })?;

  match relative.to_str() {
    Some(name) => Ok(normalize_archive_name(name)),
    None => Err(XrfError::new_encoding_error(format!(
      "Packing source '{}' cannot be named in an archive: its host path is not valid Unicode",
      format_path(path)
    ))),
  }
}

/// Spell an authored name the way an archive stores one: X-Ray separators, no leading or trailing one.
pub(crate) fn normalize_archive_name(name: &str) -> String {
  name.replace('/', "\\").trim_matches('\\').to_string()
}

/// Checks that a caller-supplied name is one host file name rather than a path.
///
/// A volume is published as `destination.join(name)`, and `join` honours a rooted or separator-bearing name by
/// dropping the destination the caller was shown. Both separators and the drive colon are refused on every host,
/// because `std::path` on Linux reads `..\outside` and `C:\outside` as one ordinary component, which would carry
/// the escape to the host that recognizes it.
pub(crate) fn validate_host_file_name(name: &str, what: &str) -> XrfResult {
  // With neither separator nor colon left, a name is one component or it is empty, `.`, or `..`.
  let is_one_file_name: bool = !name.contains(['\\', '/', ':'])
    && matches!(
      Path::new(name).components().next(),
      Some(Component::Normal(component)) if component == OsStr::new(name)
    );

  if is_one_file_name {
    Ok(())
  } else {
    Err(XrfError::new_invalid_error(format!(
      "{what} '{name}' must be a single file name, without a separator, a drive, or a traversal"
    )))
  }
}

/// Whether an engine name is a directory prefix itself or lies below it.
///
/// The boundary rule both packing and extraction need. Compared segment-wise rather than by raw `starts_with`, so
/// `configs` does not swallow `configs_backup`. Case is ignored and either separator closes a component, because a
/// volume records a name as authored and a configuration names a directory as typed, while the engine folds both.
///
/// An empty prefix names the source root, which takes everything.
pub(crate) fn is_component_prefix(name: &str, prefix: &str) -> bool {
  if prefix.is_empty() {
    return true;
  }

  let Some((head, tail)) = name.split_at_checked(prefix.len()) else {
    return false;
  };

  head.eq_ignore_ascii_case(prefix) && (tail.is_empty() || tail.starts_with(['\\', '/']))
}

/// The part of an engine name below a directory prefix, or `None` when it lies outside it.
///
/// Descendants only, unlike [`is_component_prefix`]: a name equal to the prefix leaves nothing to write below an
/// extraction destination.
pub(crate) fn relative_to_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
  if prefix.is_empty() {
    return Some(name);
  }

  if !is_component_prefix(name, prefix) {
    return None;
  }

  // The boundary is proven, so what follows the prefix is either empty or starts at the separator.
  name[prefix.len()..].strip_prefix(['\\', '/'])
}

#[cfg(test)]
mod tests {
  use std::ffi::OsString;
  use std::path::{Path, PathBuf};

  use super::{is_component_prefix, relative_to_prefix, to_archive_name, to_host_relative, validate_host_file_name};

  /// A file name the host accepts and UTF-8 cannot spell: raw bytes on Unix, a lone surrogate on Windows.
  fn unrepresentable_name() -> OsString {
    #[cfg(unix)]
    {
      use std::os::unix::ffi::OsStringExt;

      OsString::from_vec(b"system\xff.ltx".to_vec())
    }

    #[cfg(windows)]
    {
      use std::os::windows::ffi::OsStringExt;

      OsString::from_wide(&[0x0073, 0xd800, 0x002e, 0x006c, 0x0074, 0x0078])
    }
  }

  #[test]
  fn an_entry_name_crosses_into_host_components_rather_than_one_name() {
    assert_eq!(
      to_host_relative("configs\\system.ltx").expect("safe archive path"),
      Path::new("configs").join("system.ltx")
    );
    assert_eq!(
      to_host_relative("configs/system.ltx").expect("safe archive path"),
      Path::new("configs").join("system.ltx")
    );
    assert_eq!(
      to_host_relative("system.ltx").expect("safe archive path"),
      Path::new("system.ltx")
    );
    // A volume's entry point carries a trailing separator, which must not become an empty component.
    assert_eq!(
      to_host_relative("gamedata\\").expect("safe archive path"),
      Path::new("gamedata")
    );
  }

  #[test]
  fn rejects_archive_path_components_that_could_escape_the_destination() {
    for path in ["..\\system.ltx", "configs/./system.ltx", "C:/system.ltx"] {
      assert!(to_host_relative(path).is_err(), "'{path}' must not become a host path");
    }
  }

  #[test]
  fn a_source_path_becomes_the_name_the_engine_resolves() {
    let source: &Path = Path::new("gamedata");

    assert_eq!(
      to_archive_name(source, &source.join("configs").join("system.ltx")).expect("a name below the root"),
      "configs\\system.ltx"
    );
  }

  #[test]
  fn a_path_outside_the_source_root_is_refused() {
    // Only reachable if the walk left the root it was given, which is worth saying rather than passing over.
    assert!(
      to_archive_name(
        Path::new("gamedata"),
        Path::new("elsewhere").join("system.ltx").as_path()
      )
      .is_err()
    );
  }

  #[test]
  fn a_source_path_that_is_not_valid_unicode_is_refused_rather_than_dropped() {
    let source: &Path = Path::new("gamedata");
    let path: PathBuf = source.join("configs").join(unrepresentable_name());

    // There is no archive name to write for it, and dropping it would leave a successful pack one file short.
    assert!(
      to_archive_name(source, &path).is_err(),
      "an unnameable file is reported"
    );
  }

  #[test]
  fn a_published_name_may_be_one_host_file_name() {
    for name in ["gamedata", "levels.part2", "weapons-v1"] {
      assert!(
        validate_host_file_name(name, "Archive name").is_ok(),
        "'{name}' names one file"
      );
    }
  }

  #[test]
  fn rejects_a_published_name_that_would_leave_its_destination() {
    // A rooted or drive-qualified name does not escape the destination so much as discard it: `join` returns it
    // whole. The Windows spellings are refused on Linux too, where they would otherwise pack as innocent file names
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
    ] {
      assert!(
        validate_host_file_name(name, "Archive name").is_err(),
        "'{name}' must not name a published file"
      );
    }
  }

  #[test]
  fn a_prefix_matches_on_whole_segments() {
    // The trap: a raw `starts_with` would pull `configs_backup` into an extraction of `configs`.
    assert_eq!(
      relative_to_prefix("configs\\gameplay\\dialogs.xml", "configs"),
      Some("gameplay\\dialogs.xml")
    );
    assert_eq!(relative_to_prefix("configs_backup\\a.ltx", "configs"), None);
  }

  #[test]
  fn an_empty_prefix_takes_everything() {
    assert_eq!(relative_to_prefix("configs\\a.ltx", ""), Some("configs\\a.ltx"));
  }

  #[test]
  fn a_prefix_matches_neither_itself_nor_a_shorter_name() {
    assert_eq!(relative_to_prefix("configs", "configs"), None);
    assert_eq!(relative_to_prefix("a.ltx", "configs"), None);
  }

  #[test]
  fn a_prefix_ignores_case_like_the_archives_do() {
    assert_eq!(relative_to_prefix("Configs\\a.ltx", "configs"), Some("a.ltx"));
  }

  #[test]
  fn a_component_prefix_covers_the_directory_itself_and_what_is_below_it() {
    assert!(is_component_prefix("configs", "configs"));
    assert!(is_component_prefix("configs\\system.ltx", "configs"));
    assert!(is_component_prefix("configs/system.ltx", "configs"));
    assert!(is_component_prefix("configs\\weapons\\w_ak74.ltx", "configs\\weapons"));
    assert!(is_component_prefix("anything", ""), "the source root takes everything");
  }

  #[test]
  fn a_component_prefix_stops_at_the_component_boundary() {
    // The trap a raw `starts_with` walks into: a rule for `configs` reaching every sibling spelled like it.
    for name in ["configs_backup", "configs_backup\\a.ltx", "configs2", "config"] {
      assert!(!is_component_prefix(name, "configs"), "'{name}' is outside `configs`");
    }
  }

  #[test]
  fn a_component_prefix_ignores_case_and_a_multibyte_name_stays_in_bounds() {
    assert!(is_component_prefix("Configs\\System.ltx", "configs"));
    assert!(is_component_prefix("configs\\a.ltx", "Configs"));
    // Slicing at the prefix length must not land inside a character.
    assert!(!is_component_prefix("тексты", "configs"));
  }
}
