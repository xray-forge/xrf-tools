//! Names and the host paths they cross into: the only place this crate decides separators.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{HostRelativePath, format_path};

/// Either separator a name or a rule may be spelled with.
const SEPARATORS: [char; 2] = ['\\', '/'];

/// Converts an engine name into host path components below a destination root.
///
/// # Errors
///
/// Returns an invalid error when a component would leave the destination.
pub(crate) fn to_host_relative(name: &str) -> XrfResult<HostRelativePath> {
  HostRelativePath::new(name, "Archive entry path")
}

/// Express a host path as the engine name relative to the root it was found under.
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

/// Whether an engine name is a directory prefix itself or lies below it.
pub(crate) fn is_component_prefix(name: &str, prefix: &str) -> bool {
  is_root(prefix) || strip_name_prefix(name, prefix).is_some_and(|tail| tail.is_empty() || tail.starts_with(SEPARATORS))
}

/// Whether an engine name and a directory rule name the same directory.
pub(crate) fn is_same_name(name: &str, other: &str) -> bool {
  strip_name_prefix(name, other).is_some_and(|tail| tail.trim_matches(SEPARATORS).is_empty())
}

/// The part of an engine name below a directory prefix, or `None` when it lies outside it.
pub(crate) fn relative_to_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
  if is_root(prefix) {
    return Some(name);
  }

  strip_name_prefix(name, prefix)?.strip_prefix(SEPARATORS)
}

/// Whether a directory rule names the source root, which holds every name.
fn is_root(prefix: &str) -> bool {
  prefix.trim_matches(SEPARATORS).is_empty()
}

/// What follows `prefix` in `name`, compared the way an author's directory rule means it: without ASCII case, either
/// separator standing for the other, and a separator at either end of the prefix ignored. `None` when the name does not
/// open with it; the tail is not checked for a component boundary.
fn strip_name_prefix<'a>(name: &'a str, prefix: &str) -> Option<&'a str> {
  let prefix: &str = prefix.trim_matches(SEPARATORS);
  // `get` refuses a length that would cut a character, which a byte comparison below would otherwise read across.
  let head: &str = name.get(..prefix.len())?;
  let is_same: bool = head.bytes().zip(prefix.bytes()).all(|(left, right)| {
    left.eq_ignore_ascii_case(&right)
      || (SEPARATORS.contains(&char::from(left)) && SEPARATORS.contains(&char::from(right)))
  });

  is_same.then(|| &name[prefix.len()..])
}

/// Whether `path` names `root` itself or something inside it.
pub(crate) fn is_inside_directory(path: &Path, root: &Path) -> bool {
  let path: String = to_comparable(path);
  let root: String = to_comparable(root);

  path == root
    || path
      .strip_prefix(&root)
      .is_some_and(|rest| rest.starts_with(['\\', '/']))
}

/// A path in the spelling two names for one place share.
fn to_comparable(path: &Path) -> String {
  let mut missing: Vec<&OsStr> = Vec::new();
  let mut cursor: &Path = path;

  loop {
    if let Ok(resolved) = cursor.canonicalize() {
      let mut rebuilt: PathBuf = resolved;

      rebuilt.extend(missing.iter().rev());

      return spell(&rebuilt);
    }

    match (cursor.file_name(), cursor.parent()) {
      (Some(name), Some(parent)) => {
        missing.push(name);
        cursor = parent;
      }
      _ => return spell(path),
    }
  }
}

/// One separator and one case, so only the places differ.
fn spell(path: &Path) -> String {
  path.to_string_lossy().replace('/', "\\").to_lowercase()
}

#[cfg(test)]
mod tests {
  use std::ffi::OsString;
  use std::path::{Path, PathBuf};

  use super::{is_component_prefix, is_same_name, relative_to_prefix, to_archive_name, to_host_relative};

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
      to_host_relative("configs\\system.ltx")
        .expect("safe archive path")
        .as_path(),
      Path::new("configs").join("system.ltx")
    );
    // A volume's entry point carries a trailing separator, which must not become an empty component.
    assert_eq!(
      to_host_relative("gamedata\\").expect("safe archive path").as_path(),
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

  #[test]
  fn a_rule_names_its_directory_however_its_separators_are_spelled() {
    for rule in [
      "configs\\private",
      "configs/private",
      "configs\\private\\",
      "configs/private/",
      "\\configs\\private",
    ] {
      assert!(is_component_prefix("configs\\private", rule), "'{rule}'");
      assert!(is_component_prefix("configs\\private\\draft.ltx", rule), "'{rule}'");
      assert!(
        !is_component_prefix("configs\\private_backup\\draft.ltx", rule),
        "'{rule}'"
      );
      assert!(!is_component_prefix("configs\\public\\release.ltx", rule), "'{rule}'");

      assert!(is_same_name("configs\\private", rule), "'{rule}'");
      assert!(!is_same_name("configs\\private\\draft.ltx", rule), "'{rule}'");

      assert_eq!(
        relative_to_prefix("configs\\private\\draft.ltx", rule),
        Some("draft.ltx"),
        "'{rule}'"
      );
    }

    for root in ["", "\\", "/"] {
      assert!(is_component_prefix("configs\\a.ltx", root), "'{root}' is the root");
      assert_eq!(relative_to_prefix("configs\\a.ltx", root), Some("configs\\a.ltx"));
    }
  }
}
