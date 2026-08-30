use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_utils::format_path;

static STAGED_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write a complete artifact beside its destination, then move it into place.
///
/// A direct `fs::write` opens the final path with truncation, so a failure part-way through destroys
/// whatever a previous run left there. Publishing a report is exactly the case where that matters:
/// a CI job that reads the destination afterwards would take a stale or half-written document for
/// this run's answer. Staging removes the window - the destination either holds the previous
/// artifact or this one, never a mixture.
///
/// The staged file is `sync_all`ed before the rename, because a rename is atomic for the directory
/// entry while the bytes it points at may still be in the page cache. `Write::flush` on a `File` is
/// a no-op and is not that guarantee.
///
/// Unlike the translation crate's in-place saver, the destination need not already exist: a report
/// path names where a document should end up, not a file being edited.
///
/// Reports the raw [`IoError`] so a caller can name the artifact it was publishing while keeping the
/// original [`ErrorKind`].
pub fn write_file_staged(path: &Path, contents: &[u8]) -> IoResult<()> {
  let sequence: u64 = STAGED_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
  let staged_path: PathBuf = get_sibling_path(path, sequence)?;

  let result: IoResult<()> = (|| -> IoResult<()> {
    let mut staged_file: File = OpenOptions::new().write(true).create_new(true).open(&staged_path)?;

    staged_file.write_all(contents)?;

    #[cfg(test)]
    if let Some(error) = faults::take_injected_failure() {
      return Err(error);
    }

    // Only a replacement inherits permissions; a new artifact takes the platform default.
    if let Ok(metadata) = fs::metadata(path) {
      staged_file.set_permissions(metadata.permissions())?;
    }

    staged_file.sync_all()?;
    drop(staged_file);

    // One rename on every platform: `fs::rename` replaces an existing destination everywhere Rust
    // supports, Windows included.
    fs::rename(&staged_path, path)
  })();

  if result.is_err() {
    let _ = fs::remove_file(&staged_path);
  }

  result
}

/// A hidden sibling of `path`, named after it so a leftover says which file it belonged to.
///
/// Built by pushing onto the real `OsStr` rather than formatting `to_string_lossy` into a string, so
/// a destination whose name is not valid Unicode still stages under a name derived from it. The
/// process id and sequence make it unique regardless of that.
fn get_sibling_path(path: &Path, sequence: u64) -> IoResult<PathBuf> {
  let parent: &Path = path.parent().ok_or_else(|| {
    IoError::new(
      ErrorKind::InvalidInput,
      format!("File has no parent directory: {}", format_path(path)),
    )
  })?;
  let file_name: &OsStr = path.file_name().ok_or_else(|| {
    IoError::new(
      ErrorKind::InvalidInput,
      format!("File has no name: {}", format_path(path)),
    )
  })?;

  let mut staged_name: OsString = OsString::from(".");

  staged_name.push(file_name);
  staged_name.push(format!(".xrf-tmp-{}-{sequence}", process::id()));

  Ok(parent.join(staged_name))
}

/// The publication seam a test fails on purpose.
///
/// Test-only, so the shipped binary carries neither the flag nor the branch reading it.
#[cfg(test)]
pub mod faults {
  use std::cell::Cell;
  use std::io::{Error as IoError, ErrorKind};

  thread_local! {
    /// Per-thread, so tests running in parallel cannot arm each other's failure.
    static NEXT_WRITE_FAILS: Cell<bool> = const { Cell::new(false) };
  }

  /// Arm the next staged write on this thread to fail as a full device would.
  pub fn fail_next_staged_write() {
    NEXT_WRITE_FAILS.with(|armed| armed.set(true));
  }

  /// Consume the arming, so one call fails and the next publishes normally.
  pub(super) fn take_injected_failure() -> Option<IoError> {
    NEXT_WRITE_FAILS
      .with(|armed| armed.replace(false))
      .then(|| IoError::new(ErrorKind::StorageFull, "Injected staging failure"))
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::io::Result as IoResult;
  use std::path::{Path, PathBuf};

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::faults::fail_next_staged_write;
  use super::write_file_staged;

  /// An empty directory of this run's own, since every case here is about what a destination holds.
  fn get_test_directory(name: &str) -> IoResult<PathBuf> {
    let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("core/staged_write/{name}"));

    fs::create_dir_all(&directory)?;

    Ok(directory)
  }

  #[test]
  fn creates_a_destination_that_does_not_exist_yet() -> IoResult<()> {
    let directory: PathBuf = get_test_directory("creates")?;
    let path: PathBuf = directory.join("report.json");

    write_file_staged(&path, b"{}")?;

    assert_eq!(fs::read(&path)?, b"{}");

    Ok(())
  }

  #[test]
  fn replaces_an_existing_destination() -> IoResult<()> {
    let directory: PathBuf = get_test_directory("replaces")?;
    let path: PathBuf = directory.join("report.json");

    fs::write(&path, b"{\"previous\":true}")?;
    write_file_staged(&path, b"{\"current\":true}")?;

    assert_eq!(fs::read(&path)?, b"{\"current\":true}");

    Ok(())
  }

  #[test]
  fn leaves_no_staging_files_behind() -> IoResult<()> {
    let directory: PathBuf = get_test_directory("leftovers")?;
    let path: PathBuf = directory.join("report.json");

    write_file_staged(&path, b"{}")?;

    let names: Vec<String> = fs::read_dir(&directory)?
      .filter_map(Result::ok)
      .map(|entry| entry.file_name().to_string_lossy().to_string())
      .collect();

    assert_eq!(names, vec![String::from("report.json")]);

    Ok(())
  }

  #[test]
  fn keeps_the_previous_artifact_when_staging_fails() -> IoResult<()> {
    let directory: PathBuf = get_test_directory("refused")?;
    let path: PathBuf = directory.join("report.json");

    fs::write(&path, b"{\"previous\":true}")?;

    // A destination inside a directory that does not exist cannot be staged.
    let unreachable: PathBuf = directory.join("missing").join("report.json");

    assert!(write_file_staged(&unreachable, b"{}").is_err());
    assert!(!unreachable.exists(), "a refused write must not create the target");
    assert_eq!(fs::read(&path)?, b"{\"previous\":true}");

    Ok(())
  }

  #[test]
  fn keeps_the_previous_artifact_when_the_staged_write_fails() -> IoResult<()> {
    let directory: PathBuf = get_test_directory("injected")?;
    let path: PathBuf = directory.join("report.json");

    fs::write(&path, b"{\"previous\":true}")?;
    fail_next_staged_write();

    assert!(write_file_staged(&path, b"{\"current\":true}").is_err());
    assert_eq!(fs::read(&path)?, b"{\"previous\":true}");

    // The staged file is removed on the way out, so a failed run leaves the directory as it was.
    let names: Vec<String> = fs::read_dir(&directory)?
      .filter_map(Result::ok)
      .map(|entry| entry.file_name().to_string_lossy().to_string())
      .collect();

    assert_eq!(names, vec![String::from("report.json")]);

    // The arming is consumed, so the next publication succeeds.
    write_file_staged(&path, b"{\"current\":true}")?;

    assert_eq!(fs::read(&path)?, b"{\"current\":true}");

    Ok(())
  }

  #[test]
  fn refuses_a_path_that_names_no_file() {
    assert!(write_file_staged(Path::new(""), b"{}").is_err());
  }
}
