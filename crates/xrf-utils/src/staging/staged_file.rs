use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::host_path::format_path;
#[cfg(feature = "staging-faults")]
use crate::staging::StagingFaults;

/// Numbers the siblings this process stages, so two of one file never share a name.
static SIBLING_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A file's next content written and synced in a sibling of it, not yet in its place: the first half of a staged
/// write, for a caller that publishes several files together. Removed when dropped unpublished.
#[derive(Debug)]
pub struct StagedFile {
  target: PathBuf,
  staged: PathBuf,
  is_published: bool,
}

impl StagedFile {
  /// Writes a complete file beside its destination, then moves it into place.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the staged file cannot be created, written, synced, or moved into place.
  pub fn write(path: &Path, contents: &[u8]) -> IoResult<()> {
    Self::stage(path, &Self::to_sibling(path, "tmp")?, contents)?.publish()
  }

  /// Writes a file as [`Self::write`] does, creating the directories above it where they are missing.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the directories cannot be created, or for anything [`Self::write`] refuses.
  pub fn write_creating_directories(path: &Path, contents: &[u8]) -> IoResult<()> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent)?;
    }

    Self::write(path, contents)
  }

  /// Writes and syncs `contents` at `staged`, a name from [`Self::to_sibling`], to replace `target` once published; a
  /// replacement inherits `target`'s permissions.
  ///
  /// # Errors
  ///
  /// Returns an IO error, with nothing left at `staged`, when it cannot be created, written or synced.
  pub fn stage(target: &Path, staged: &Path, contents: &[u8]) -> IoResult<Self> {
    let result: IoResult<()> = (|| -> IoResult<()> {
      let mut staged_file: File = OpenOptions::new().write(true).create_new(true).open(staged)?;

      staged_file.write_all(contents)?;

      #[cfg(feature = "staging-faults")]
      if let Some(error) = StagingFaults::take_write_failure() {
        return Err(error);
      }

      // Only a replacement inherits permissions; a new file takes the platform default.
      if let Ok(metadata) = fs::metadata(target) {
        staged_file.set_permissions(metadata.permissions())?;
      }

      staged_file.sync_all()
    })();

    if let Err(error) = result {
      let _ = fs::remove_file(staged);

      return Err(error);
    }

    Ok(Self {
      target: target.to_path_buf(),
      staged: staged.to_path_buf(),
      is_published: false,
    })
  }

  /// A unique hidden sibling of `path` for `purpose` (`tmp`, `staged`, `replaced`), named `.<name>.xrf-<purpose>-…` so a
  /// leftover says which file it belonged to and [`Self::is_sibling`] knows it.
  ///
  /// # Errors
  ///
  /// Returns an invalid-input error when `path` names no file in a directory.
  pub fn to_sibling(path: &Path, purpose: &str) -> IoResult<PathBuf> {
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
    let sequence: u64 = SIBLING_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let mut sibling_name: OsString = OsString::from(".");

    sibling_name.push(file_name);
    sibling_name.push(format!(".xrf-{purpose}-{}-{sequence}", process::id()));

    Ok(parent.join(sibling_name))
  }

  /// Whether a file name is one a staged write stages or sets aside beside a target: hidden, carrying `.xrf-`.
  pub fn is_sibling(name: &OsStr) -> bool {
    name
      .to_str()
      .is_some_and(|name| name.starts_with('.') && name.contains(".xrf-"))
  }

  /// Flushes the directory entry a rename produced, so the replacement survives a power loss too.
  ///
  /// Windows has no directory handle to sync and its renames are already ordered, so there is nothing to do there.
  pub fn sync_parent_directory(path: &Path) {
    if cfg!(windows) {
      return;
    }

    if let Some(parent) = path.parent()
      && let Err(error) = File::open(parent).and_then(|directory| directory.sync_all())
    {
      log::debug!(
        "Could not sync directory '{}' after replacing a file: {error}",
        format_path(parent)
      );
    }
  }

  pub fn get_target(&self) -> &Path {
    &self.target
  }

  pub fn get_staged_path(&self) -> &Path {
    &self.staged
  }

  /// Moves the staged content over its target in one rename, so the target holds the old content or the new one at
  /// every instant, then syncs the directory entry.
  ///
  /// # Errors
  ///
  /// Returns an IO error, with the staged file removed and the target as it was, when the rename fails.
  pub fn publish(mut self) -> IoResult<()> {
    #[cfg(feature = "staging-faults")]
    if let Some(error) = StagingFaults::take_publish_failure() {
      return Err(error);
    }

    // One rename on every platform: `fs::rename` replaces an existing destination everywhere Rust supports, Windows
    // included, through `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`.
    fs::rename(&self.staged, &self.target)?;

    self.is_published = true;

    Self::sync_parent_directory(&self.target);

    Ok(())
  }
}

impl Drop for StagedFile {
  fn drop(&mut self) {
    if !self.is_published {
      let _ = fs::remove_file(&self.staged);
    }
  }
}
