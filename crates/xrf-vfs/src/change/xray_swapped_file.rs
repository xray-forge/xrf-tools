use std::fs::{self, File};
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::change::xray_journal_entry::XrayJournalEntry;

/// One file a commit swaps: its target, the new content staged beside it, and where the file it replaces is kept.
#[derive(Debug)]
pub(crate) struct XraySwappedFile {
  pub(crate) target: PathBuf,
  /// The new content beside the target, `None` for a delete.
  pub(crate) staged: Option<StagedFile>,
  /// Where the file the target held is kept until the commit is done, for one that was there.
  pub(crate) backup: Option<PathBuf>,
}

impl XraySwappedFile {
  /// The file as the commit's journal names it.
  pub(crate) fn to_entry(&self) -> XrayJournalEntry {
    XrayJournalEntry {
      target: self.target.clone(),
      staged: self
        .staged
        .as_ref()
        .map(|staged| staged.get_staged_path().to_path_buf()),
      backup: self.backup.clone(),
    }
  }

  /// Keeps the file the target holds, then moves the staged content over it, or deletes it.
  ///
  /// # Errors
  ///
  /// Returns an error when the kept copy cannot be made, a delete names a file not on disk, or the move fails.
  pub(crate) fn swap(&mut self) -> XrfResult<()> {
    if let Some(backup) = &self.backup {
      Self::keep(&self.target, backup)?;
    } else if self.staged.is_none() {
      return Err(XrfError::new_asset_error(format!(
        "cannot delete '{}': it is not on disk",
        format_path(&self.target)
      )));
    }

    match self.staged.take() {
      Some(staged) => staged.publish(),
      None => fs::remove_file(&self.target),
    }
    .map_err(|error| XrfError::new_asset_error(format!("failed to write '{}': {error}", format_path(&self.target))))
  }

  /// A second name for the file at `target`, so it survives being replaced: a hard link where the volume takes one, else
  /// a synced copy.
  fn keep(target: &Path, backup: &Path) -> XrfResult<()> {
    if fs::hard_link(target, backup).is_ok() {
      return Ok(());
    }

    fs::copy(target, backup)
      .and_then(|_| File::open(backup)?.sync_all())
      .map_err(|error| XrfError::new_asset_error(format!("failed to keep '{}': {error}", format_path(target))))
  }
}
