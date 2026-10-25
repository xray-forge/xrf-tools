/// Splitting the path a document source names a config by, as named operations.
///
/// A source's paths are logical ones, backslash separated, out of a VFS, or host paths for a file read by path, which
/// separate with `/` on Linux and wherever a caller wrote them so on Windows. Both separators split, as
/// [`xrf_ltx::Ltx::directory_of`] splits them, so the directory a config's patches are listed in and its includes are
/// resolved from is the one it sits in whichever kind of path named it. Not `XrayLogicalPath`, which would normalise a
/// host path into an engine one.
pub(crate) struct DltxSourcePath;

impl DltxSourcePath {
  const SEPARATORS: [char; 2] = ['\\', '/'];

  /// The last separated segment, or the whole path when it has no separator.
  pub(crate) fn file_name_of(path: &str) -> &str {
    match path.rfind(Self::SEPARATORS) {
      Some(index) => &path[index + 1..],
      None => path,
    }
  }

  /// Everything before the last separator, or the empty string for a top-level config.
  pub(crate) fn directory_of(path: &str) -> &str {
    match path.rfind(Self::SEPARATORS) {
      Some(index) => &path[..index],
      None => "",
    }
  }

  /// The file `name` beside `path`, in its directory and written with the separator `path` already uses.
  pub(crate) fn sibling_of(path: &str, name: &str) -> String {
    match path.rfind(Self::SEPARATORS) {
      Some(index) => format!("{}{name}", &path[..=index]),
      None => String::from(name),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::DltxSourcePath;

  #[test]
  fn splits_a_nested_path() {
    assert_eq!(
      DltxSourcePath::directory_of("configs\\items\\w_ak74.ltx"),
      "configs\\items"
    );
    assert_eq!(DltxSourcePath::file_name_of("configs\\items\\w_ak74.ltx"), "w_ak74.ltx");
  }

  #[test]
  fn a_top_level_config_has_no_directory() {
    assert_eq!(DltxSourcePath::directory_of("system.ltx"), "");
    assert_eq!(DltxSourcePath::file_name_of("system.ltx"), "system.ltx");
    assert_eq!(
      DltxSourcePath::sibling_of("system.ltx", "mod_system_a.ltx"),
      "mod_system_a.ltx"
    );
  }

  #[test]
  fn splits_a_host_path_at_either_separator() {
    // A file read by path names itself as the host does, which on Linux is never a backslash.
    assert_eq!(
      DltxSourcePath::directory_of("/games/configs/system.ltx"),
      "/games/configs"
    );
    assert_eq!(
      DltxSourcePath::file_name_of("C:/games\\configs/system.ltx"),
      "system.ltx"
    );
  }

  #[test]
  fn names_a_sibling_with_the_separator_the_path_uses() {
    // A backslash appended to a Linux path is part of a file name there, not a separator.
    assert_eq!(
      DltxSourcePath::sibling_of("/games/configs/system.ltx", "mod_system_a.ltx"),
      "/games/configs/mod_system_a.ltx"
    );
    assert_eq!(
      DltxSourcePath::sibling_of("configs\\system.ltx", "mod_system_a.ltx"),
      "configs\\mod_system_a.ltx"
    );
  }
}
