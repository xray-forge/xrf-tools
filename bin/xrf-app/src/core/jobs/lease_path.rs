use std::path::{Path, PathBuf};

/// One spelling of a host path, for deciding whether two jobs would collide on it.
///
/// Canonicalized where the path exists and left lexically as given where it does not, because a destination is
/// commonly typed before it is created and `canonicalize` refuses a path that is not there yet. Two spellings of an
/// existing directory therefore compare equal as they should; two spellings of one that does not exist yet may not,
/// which is the narrow gap this leaves open.
///
/// Lower-cased because the hosts these tools run on treat paths case-insensitively, and a lease that let `C:\Out` and
/// `c:\out` run at once would be no lease at all. That is wrong on a case-sensitive filesystem, where it refuses two
/// runs that could have coexisted — the safe direction for an exclusion rule to be wrong in.
pub fn to_comparable_path(path: &Path) -> String {
  let resolved: PathBuf = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

  resolved.to_string_lossy().to_lowercase()
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::to_comparable_path;

  #[test]
  fn two_spellings_of_one_place_compare_equal() {
    assert_eq!(
      to_comparable_path(Path::new("C:\\Out\\GameData")),
      to_comparable_path(Path::new("c:\\out\\gamedata"))
    );
  }

  #[test]
  fn different_places_stay_different() {
    assert_ne!(
      to_comparable_path(Path::new("C:\\out\\a")),
      to_comparable_path(Path::new("C:\\out\\b"))
    );
  }
}
