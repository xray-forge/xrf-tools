use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// Files a commit replaces, creates or deletes on disk together: every new content staged beside its target first, then
/// every target swapped, and every swap undone when one fails, so the files hold either all of a commit or none of it.
#[derive(Debug, Default)]
pub(crate) struct XrayDiskSwap {
  files: Vec<XraySwappedFile>,
}

#[derive(Debug)]
struct XraySwappedFile {
  target: PathBuf,
  /// The new content beside the target, `None` for a delete.
  staged: Option<PathBuf>,
  /// Where the file the target held was moved, once it was.
  aside: Option<PathBuf>,
  is_placed: bool,
}

impl XrayDiskSwap {
  /// Stages `bytes` beside `target`, creating the directories above it, or, for `None`, marks `target` to be deleted.
  ///
  /// # Errors
  ///
  /// Returns an error when a directory or the staged file cannot be created, written or synced. What was staged
  /// before is removed when the swap is dropped.
  pub(crate) fn stage(&mut self, target: PathBuf, bytes: Option<&[u8]>) -> XrfResult<()> {
    let staged: Option<PathBuf> = match bytes {
      Some(bytes) => Some(Self::write_beside(&target, bytes)?),
      None => None,
    };

    self.files.push(XraySwappedFile {
      target,
      staged,
      aside: None,
      is_placed: false,
    });

    Ok(())
  }

  /// Swaps every staged file into place and deletes every marked one, undoing all of it when one fails.
  ///
  /// # Errors
  ///
  /// Returns the first failure, after putting back every file swapped before it.
  pub(crate) fn swap(mut self) -> XrfResult<()> {
    for index in 0..self.files.len() {
      if let Err(error) = self.files[index].swap() {
        self.undo();

        return Err(error);
      }
    }

    // Best effort: the swap has happened, and a leftover beside a file is the cost of a failure to tidy after it.
    for file in &mut self.files {
      if let Some(aside) = file.aside.take()
        && let Err(error) = fs::remove_file(&aside)
      {
        log::warn!("Could not remove {} after a commit: {error}", format_path(&aside));
      }
    }

    Ok(())
  }

  /// Puts every swapped file back, in reverse order.
  fn undo(&mut self) {
    for file in self.files.iter_mut().rev() {
      file.undo();
    }
  }

  /// Writes and syncs `bytes` in a hidden sibling of `target`, named after it so a leftover says which file it was for.
  fn write_beside(target: &Path, bytes: &[u8]) -> XrfResult<PathBuf> {
    if let Some(parent) = target.parent() {
      fs::create_dir_all(parent)
        .map_err(|error| XrfError::new_asset_error(format!("failed to create '{}': {error}", format_path(parent))))?;
    }

    let staged: PathBuf = Self::get_sibling(target, "staged")?;
    let result: std::io::Result<()> = (|| {
      let mut file: File = OpenOptions::new().write(true).create_new(true).open(&staged)?;

      file.write_all(bytes)?;

      if let Ok(metadata) = fs::metadata(target) {
        file.set_permissions(metadata.permissions())?;
      }

      file.sync_all()
    })();

    if let Err(error) = result {
      let _ = fs::remove_file(&staged);

      return Err(XrfError::new_asset_error(format!(
        "failed to stage '{}': {error}",
        format_path(target)
      )));
    }

    Ok(staged)
  }

  fn get_sibling(target: &Path, purpose: &str) -> XrfResult<PathBuf> {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    let (Some(parent), Some(name)) = (target.parent(), target.file_name()) else {
      return Err(XrfError::new_asset_error(format!(
        "cannot write '{}': it names no file in a directory",
        format_path(target)
      )));
    };

    let mut sibling: OsString = OsString::from(".");

    sibling.push(name);
    sibling.push(format!(
      ".xrf-{purpose}-{}-{}",
      process::id(),
      SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));

    Ok(parent.join(sibling))
  }
}

impl XraySwappedFile {
  /// Moves the file the target holds aside, then the staged content into its place.
  fn swap(&mut self) -> XrfResult<()> {
    if self.target.exists() {
      let aside: PathBuf = XrayDiskSwap::get_sibling(&self.target, "replaced")?;

      fs::rename(&self.target, &aside).map_err(|error| {
        XrfError::new_asset_error(format!("failed to replace '{}': {error}", format_path(&self.target)))
      })?;

      self.aside = Some(aside);
    } else if self.staged.is_none() {
      return Err(XrfError::new_asset_error(format!(
        "cannot delete '{}': it is not on disk",
        format_path(&self.target)
      )));
    }

    if let Some(staged) = &self.staged {
      fs::rename(staged, &self.target).map_err(|error| {
        XrfError::new_asset_error(format!("failed to write '{}': {error}", format_path(&self.target)))
      })?;

      self.is_placed = true;
    }

    Ok(())
  }

  /// Puts back what [`Self::swap`] moved, as far as it got.
  fn undo(&mut self) {
    if self.is_placed {
      let _ = fs::remove_file(&self.target);

      self.is_placed = false;
    }

    if let Some(aside) = self.aside.take()
      && let Err(error) = fs::rename(&aside, &self.target)
    {
      log::error!(
        "Could not restore {} from {} after a failed commit: {error}",
        format_path(&self.target),
        format_path(&aside)
      );
    }
  }
}

impl Drop for XrayDiskSwap {
  /// Removes whatever is still staged: everything, when the swap never ran or failed.
  fn drop(&mut self) {
    for file in &self.files {
      if let Some(staged) = &file.staged
        && !file.is_placed
      {
        let _ = fs::remove_file(staged);
      }
    }
  }
}
