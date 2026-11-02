use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

/// Where a packed-file command writes: refused while it is there unless the caller forced it, and replaced only once
/// the new one is complete, so a source that cannot be read leaves the old output standing.
pub struct PackedFileDestination;

impl PackedFileDestination {
  /// Refuses `dest` when it is already there and the caller did not force replacing it.
  ///
  /// # Errors
  ///
  /// Returns an already-exists error naming `--force`.
  pub fn require_free(dest: &Path, is_forced: bool) -> XrfResult {
    if dest.exists() && !is_forced {
      return Err(XrfError::new_io_error(
        format!("{} already exists, use --force to replace it", format_path(dest)),
        std::io::ErrorKind::AlreadyExists,
      ));
    }

    Ok(())
  }

  /// Writes a folder at `dest` through `export`: straight into it where nothing is there, else into a staged sibling
  /// that replaces the old folder once complete.
  ///
  /// # Errors
  ///
  /// Returns an error when the export fails, or the old folder cannot be replaced.
  pub fn write_folder(dest: &Path, export: impl FnOnce(&Path) -> XrfResult) -> XrfResult {
    if !dest.exists() {
      return export(dest);
    }

    let staged: PathBuf = StagedFile::to_sibling(dest, "staged")?;

    if let Err(error) = export(&staged) {
      let _ = fs::remove_dir_all(&staged);

      return Err(error);
    }

    if dest.is_dir() {
      fs::remove_dir_all(dest)?;
    } else {
      fs::remove_file(dest)?;
    }

    Ok(fs::rename(&staged, dest)?)
  }
}
