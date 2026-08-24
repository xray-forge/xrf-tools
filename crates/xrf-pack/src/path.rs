//! Engine names and the host paths they cross into: the only place this crate decides separators.

use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};

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

/// The part of an engine name below a directory prefix, or `None` when it lies outside it.
///
/// Compared segment-wise rather than by raw `starts_with`, so `configs` does not swallow `configs_backup\...`. Case is
/// ignored, because a volume records a name as authored and the engine resolves it either way.
pub(crate) fn relative_to_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
  if prefix.is_empty() {
    return Some(name);
  }

  if name.len() <= prefix.len() {
    return None;
  }

  let (head, tail) = name.split_at(prefix.len());

  if head.eq_ignore_ascii_case(prefix) && tail.starts_with(['\\', '/']) {
    Some(&tail[1..])
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::{relative_to_prefix, to_host_relative};

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
}
