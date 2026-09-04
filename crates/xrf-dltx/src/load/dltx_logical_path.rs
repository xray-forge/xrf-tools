//! Splitting an X-Ray logical path, which the load pass does for every file it reads.

/// The separator X-Ray logical paths use, whatever the host's own is.
const SEPARATOR: char = '\\';

/// The last separated segment, or the whole path when it has no separator.
pub fn file_name_of(logical_path: &str) -> &str {
  match logical_path.rsplit_once(SEPARATOR) {
    Some((_, name)) => name,
    None => logical_path,
  }
}

/// Everything before the last separator, or the empty string for a top-level config.
pub fn directory_of(logical_path: &str) -> &str {
  match logical_path.rsplit_once(SEPARATOR) {
    Some((directory, _)) => directory,
    None => "",
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn splits_a_nested_path() {
    assert_eq!(directory_of("configs\\items\\w_ak74.ltx"), "configs\\items");
    assert_eq!(file_name_of("configs\\items\\w_ak74.ltx"), "w_ak74.ltx");
  }

  #[test]
  fn a_top_level_config_has_no_directory() {
    assert_eq!(directory_of("system.ltx"), "");
    assert_eq!(file_name_of("system.ltx"), "system.ltx");
  }
}
