use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{Error as IoError, ErrorKind, Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::path_utils::format_path;

static STAGED_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Write a complete file beside its destination, then move it into place.
///
/// A direct `fs::write` opens the final path with truncation, so a failure part-way through destroys whatever a
/// previous run left there. Every artifact this workspace publishes over a file a person or a later run reads back —
/// a report, an extern manifest, a saved translation, a packing configuration — is exactly the case where that
/// matters. Staging removes the window: the destination holds either the previous file or this one, never a mixture.
///
/// Durable against the machine losing power, not only against this process failing. The staged file is `sync_all`ed
/// before the rename, because a rename is atomic with respect to the directory entry while the bytes it points at may
/// still be sitting in the page cache: the crash window between them is where a file becomes present but empty.
/// `Write::flush` is not that guarantee — on a `File` it is a no-op, since `write_all` has already reached the kernel.
///
/// The destination need not already exist; only a replacement inherits permissions, since a file that is not there
/// cannot supply them. The directories above it must, so a path into a missing directory is refused rather than
/// created — use [`write_new_file_staged`] where creating them is the intent.
///
/// Reports the raw [`IoError`] so a caller can name the artifact it was publishing while keeping the original
/// [`ErrorKind`].
///
/// # Errors
///
/// Returns an IO error when the staged file cannot be created, written, synced, or moved into place.
pub fn write_file_staged(path: &Path, contents: &[u8]) -> IoResult<()> {
  write_staged(path, contents, false)
}

/// Write a file that is not there yet, creating the directories above it.
///
/// Staged the same way regardless: an importer writing a hundred files should not leave a half-written one behind when
/// the disk fills on the ninety-ninth.
///
/// # Errors
///
/// Returns an IO error when the directories cannot be created, or for anything [`write_file_staged`] refuses.
pub fn write_new_file_staged(path: &Path, contents: &[u8]) -> IoResult<()> {
  write_staged(path, contents, true)
}

fn write_staged(path: &Path, contents: &[u8], is_creating_directories: bool) -> IoResult<()> {
  if is_creating_directories && let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }

  let sequence: u64 = STAGED_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
  let staged_path: PathBuf = get_sibling_path(path, sequence)?;

  let result: IoResult<()> = (|| -> IoResult<()> {
    let mut staged_file: File = OpenOptions::new().write(true).create_new(true).open(&staged_path)?;

    staged_file.write_all(contents)?;

    #[cfg(feature = "staging-faults")]
    if let Some(error) = staging_faults::take_injected_failure() {
      return Err(error);
    }

    // Only a replacement inherits permissions; a new file takes the platform default.
    if let Ok(metadata) = fs::metadata(path) {
      staged_file.set_permissions(metadata.permissions())?;
    }

    staged_file.sync_all()?;
    drop(staged_file);

    // One rename on every platform: `fs::rename` replaces an existing destination everywhere Rust supports, Windows
    // included, through `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`.
    fs::rename(&staged_path, path)?;

    sync_parent_directory(path);

    Ok(())
  })();

  if result.is_err() {
    let _ = fs::remove_file(&staged_path);
  }

  result
}

/// Flush the directory entry the rename produced, so the replacement survives a power loss too.
///
/// Best effort, and intentionally not an error: the bytes are already durable by this point, so a directory that
/// cannot be synced costs at most the file reverting to its previous contents — while failing here would report a
/// write that did happen as one that did not.
///
/// Windows has no directory handle to sync and its renames are already ordered, so there is nothing to do there.
fn sync_parent_directory(path: &Path) {
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

/// A hidden sibling of `path`, named after it so a leftover says which file it belonged to.
///
/// Built by pushing onto the real `OsStr` rather than formatting `to_string_lossy` into a string, so a destination
/// whose name is not valid Unicode still stages under a name derived from it. The process id and sequence make it
/// unique regardless of that.
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
/// Behind a feature rather than `#[cfg(test)]`, because the tests that arm it live in the crates that publish
/// artifacts rather than here. A normal build turns it on nowhere, so neither the flag nor the branch reading it
/// reaches a shipped binary.
#[cfg(feature = "staging-faults")]
pub mod staging_faults {
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
mod tests;
