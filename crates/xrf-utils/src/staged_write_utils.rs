use std::ffi::{OsStr, OsString};
use std::fs::{self, File};
use std::io::{Error as IoError, ErrorKind, Result as IoResult};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::path_utils::format_path;

mod staged_file;

pub use staged_file::StagedFile;

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

/// Write a file, creating the directories above it where they are missing, and replacing the file where it exists.
///
/// Staged the same way as [`write_file_staged`]: an importer writing a hundred files should not leave a half-written
/// one behind when the disk fills on the ninety-ninth.
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

  StagedFile::stage(path, &to_staged_sibling(path, "tmp")?, contents)?.publish()
}

/// A unique hidden sibling of `path` for `purpose` (`tmp`, `staged`, `replaced`), named `.<name>.xrf-<purpose>-…` so a
/// leftover says which file it belonged to and [`is_staged_sibling`] knows it.
///
/// # Errors
///
/// Returns an invalid-input error when `path` names no file in a directory.
pub fn to_staged_sibling(path: &Path, purpose: &str) -> IoResult<PathBuf> {
  get_sibling_path(path, purpose, STAGED_FILE_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Whether a file name is one this module stages or sets aside beside a target: hidden, carrying `.xrf-`.
pub fn is_staged_sibling(name: &OsStr) -> bool {
  name
    .to_str()
    .is_some_and(|name| name.starts_with('.') && name.contains(".xrf-"))
}

/// Flush the directory entry a rename produced, so the replacement survives a power loss too.
///
/// Best effort, and intentionally not an error: the bytes are already durable by this point, so a directory that
/// cannot be synced costs at most the file reverting to its previous contents — while failing here would report a
/// write that did happen as one that did not.
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

/// A hidden sibling of `path`, named after it so a leftover says which file it belonged to.
///
/// Built by pushing onto the real `OsStr` rather than formatting `to_string_lossy` into a string, so a destination
/// whose name is not valid Unicode still stages under a name derived from it. The process id and sequence make it
/// unique regardless of that.
fn get_sibling_path(path: &Path, purpose: &str, sequence: u64) -> IoResult<PathBuf> {
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
  staged_name.push(format!(".xrf-{purpose}-{}-{sequence}", process::id()));

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
    /// How many publishes on this thread succeed before one fails, when armed.
    static PUBLISHES_BEFORE_FAILURE: Cell<Option<u32>> = const { Cell::new(None) };
  }

  /// Arm the next staged write on this thread to fail as a full device would.
  pub fn fail_next_staged_write() {
    NEXT_WRITE_FAILS.with(|armed| armed.set(true));
  }

  /// Arm a publish on this thread to fail once `succeeding` more have succeeded, as a rename refused by a locked file
  /// would.
  pub fn fail_staged_publish_after(succeeding: u32) {
    PUBLISHES_BEFORE_FAILURE.with(|armed| armed.set(Some(succeeding)));
  }

  /// Consume the arming, so one call fails and the next publishes normally.
  pub(super) fn take_injected_failure() -> Option<IoError> {
    NEXT_WRITE_FAILS
      .with(|armed| armed.replace(false))
      .then(|| IoError::new(ErrorKind::StorageFull, "Injected staging failure"))
  }

  /// Counts one publish down, failing it where the arming has run out.
  pub(super) fn take_injected_publish_failure() -> Option<IoError> {
    PUBLISHES_BEFORE_FAILURE.with(|armed| match armed.get() {
      Some(0) => {
        armed.set(None);

        Some(IoError::new(ErrorKind::PermissionDenied, "Injected publish failure"))
      }
      Some(remaining) => {
        armed.set(Some(remaining - 1));

        None
      }
      None => None,
    })
  }
}

#[cfg(test)]
mod tests;
