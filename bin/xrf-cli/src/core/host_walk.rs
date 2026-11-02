use std::path::Path;

use walkdir::WalkDir;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::format_path;

/// The files of one kind below a host path, every directory a sweep could not walk kept as an error in its place: a
/// sweep that skipped what it could not read would pass over files it never saw.
pub struct HostWalk;

impl HostWalk {
  /// `root` itself where it is a file, else every file below it `extension` names, by name so a run is repeatable;
  /// each directory or entry the walk could not read is an error naming it.
  pub fn files_with(root: &Path, extension: XrayExtension) -> Vec<XrfResult<std::path::PathBuf>> {
    if root.is_file() {
      return vec![Ok(root.to_path_buf())];
    }

    WalkDir::new(root)
      .sort_by_file_name()
      .into_iter()
      .filter_map(|entry| match entry {
        Ok(entry) => {
          (entry.file_type().is_file() && extension.matches_path(entry.path())).then(|| Ok(entry.into_path()))
        }
        Err(error) => {
          let path: String = format_path(error.path().unwrap_or(root)).to_string();
          let kind: std::io::ErrorKind = error.io_error().map_or(std::io::ErrorKind::Other, |error| error.kind());

          Some(Err(XrfError::new_io_error(
            format!("Failed to walk {path}: {error}"),
            kind,
          )))
        }
      })
      .collect()
  }
}
