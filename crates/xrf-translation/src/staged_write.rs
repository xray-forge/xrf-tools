use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions, Permissions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_error::{XrfError, XrfResult};

static STAGED_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Replace an existing file only after its complete contents are on disk beside it.
///
/// Durable against the machine losing power, not only against this process failing. The staged file is
/// `sync_all`ed before the rename, because a rename is atomic with respect to the directory entry
/// while the bytes it points at may still be sitting in the page cache: the crash window between them
/// is where a file becomes present but empty. Translator work is the thing being written, so the cost
/// of one flush per save is not worth trading for it.
///
/// `Write::flush` is not that guarantee and used to stand here in its place — on a `File` it is a
/// no-op, since `write_all` has already reached the kernel.
pub(crate) fn write_file_staged(path: &Path, contents: &[u8]) -> XrfResult {
  let sequence: u64 = STAGED_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
  let staged_path: PathBuf = sibling_path(path, "xrf-tmp", sequence)?;

  let result: XrfResult = (|| -> XrfResult {
    let permissions: Permissions = fs::metadata(path)?.permissions();
    let mut staged_file: File = OpenOptions::new().write(true).create_new(true).open(&staged_path)?;

    staged_file.write_all(contents)?;
    staged_file.set_permissions(permissions)?;
    staged_file.sync_all()?;
    drop(staged_file);

    replace_staged_file(&staged_path, path)?;
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
/// Best effort, and deliberately not an error: the bytes are already durable by this point, so a
/// directory that cannot be synced costs at most the file reverting to its previous contents — while
/// failing here would report a save that did happen as one that did not.
///
/// Windows has no directory handle to sync and `ReplaceFile`-style renames are already ordered, so
/// there is nothing to do there.
fn sync_parent_directory(path: &Path) {
  if cfg!(windows) {
    return;
  }

  if let Some(parent) = path.parent()
    && let Err(error) = File::open(parent).and_then(|directory| directory.sync_all())
  {
    log::debug!(
      "Could not sync directory '{}' after replacing a file: {error}",
      parent.display()
    );
  }
}

/// A hidden sibling of `path`, named after it so a leftover says which file it belonged to.
///
/// Built by pushing onto the real `OsStr` rather than formatting `to_string_lossy` into a string: a
/// name that is not valid Unicode would otherwise turn into replacement characters, and the staged
/// file would be created next to the target under a name that is not derived from it. The process id
/// and sequence make it unique regardless, so this is about the leftover being identifiable and the
/// crate holding one rule for host names, not two.
fn sibling_path(path: &Path, suffix: &str, sequence: u64) -> XrfResult<PathBuf> {
  let parent: &Path = path
    .parent()
    .ok_or_else(|| XrfError::new_invalid_error(format!("File has no parent directory: {}", path.display())))?;
  let file_name: &OsStr = path
    .file_name()
    .ok_or_else(|| XrfError::new_invalid_error(format!("File has no name: {}", path.display())))?;

  let mut staged_name: OsString = OsString::from(".");

  staged_name.push(file_name);
  staged_name.push(format!(".{suffix}-{}-{sequence}", std::process::id()));

  Ok(parent.join(staged_name))
}

/// Move the staged file onto the target, replacing it in one step.
///
/// One rename on every platform. `fs::rename` overwrites an existing destination everywhere Rust
/// supports — on Windows through `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`, which this crate
/// pins in `replaces_a_destination_that_is_open_for_reading`.
///
/// This replaced a Windows-only dance that renamed the target aside to a backup first, then renamed
/// the staged file into place, then deleted the backup. It defended against a restriction that does
/// not exist and paid for it: three renames instead of one, a window in which the target did not
/// exist at all, and a `.xrf-backup` file left beside the target whenever the second rename failed.
fn replace_staged_file(staged_path: &Path, path: &Path) -> XrfResult {
  Ok(fs::rename(staged_path, path)?)
}

#[cfg(test)]
mod tests;
