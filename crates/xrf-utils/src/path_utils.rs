use std::path::Path;

/// Stringify list of paths for better display in logs / debug info / information printing.
#[inline]
pub fn path_vec_to_string<T: AsRef<Path>>(paths: &[T]) -> String {
  path_vec_to_string_sep(paths, ", ")
}

/// Stringify list of paths for better display in logs / debug info / information printing.
#[inline]
pub fn path_vec_to_string_sep<T: AsRef<Path>>(paths: &[T], separator: &str) -> String {
  paths
    .iter()
    .map(|it| it.as_ref().display().to_string())
    .collect::<Vec<_>>()
    .join(separator)
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

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::to_portable_path_string;

  #[test]
  fn renders_separators_the_same_way_on_every_platform() {
    assert_eq!(
      to_portable_path_string(PathBuf::from("configs").join("gameplay").join("dialogs.xml")),
      "configs/gameplay/dialogs.xml"
    );
    assert_eq!(to_portable_path_string("already/portable"), "already/portable");
    assert_eq!(to_portable_path_string(""), "");
  }
}
