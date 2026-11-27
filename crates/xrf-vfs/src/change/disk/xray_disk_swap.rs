use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::change::disk::{XrayDiskJournal, XrayJournalEntry, XraySwappedFile};
use crate::change::transaction::XrayWriteContent;

/// Files a commit replaces, creates or deletes on disk together: every new content staged and synced beside its
/// target, a journal written beside the first target naming every file, then every target swapped, each replaced or
/// deleted file kept first as a hard link (or a copy) so the target exists at every instant, and every swap undone
/// when one fails. A crash leaves the journal, which the next mount of a directory holding it settles
/// ([`XrayDiskJournal::recover`]).
#[derive(Debug, Default)]
pub(crate) struct XrayDiskSwap {
  files: Vec<XraySwappedFile>,
  /// Directories made for new files, deepest last, removed again where the commit fails.
  directories: Vec<PathBuf>,
}

impl XrayDiskSwap {
  /// Stages `content` beside `target`, creating the directories above it, or takes the file its writer staged there;
  /// for `None`, marks `target` to be deleted.
  ///
  /// # Errors
  ///
  /// Returns an error when a directory or the staged file cannot be created, written or synced, or when a staged write
  /// was spent by an earlier commit or stands beside another file. What was staged before, the directories made for it
  /// and a staged write taken here are removed when the swap is dropped.
  pub(crate) fn stage(&mut self, target: PathBuf, content: Option<&XrayWriteContent>) -> XrfResult<()> {
    let staged: Option<StagedFile> = match content {
      Some(XrayWriteContent::Bytes(bytes)) => {
        self.create_directories_above(&target)?;

        Some(
          StagedFile::stage(&target, &StagedFile::name_sibling(&target, "staged")?, bytes).map_err(|error| {
            XrfError::new_asset_error(format!("failed to stage '{}': {error}", format_path(&target)))
          })?,
        )
      }
      Some(XrayWriteContent::Staged(slot)) => Some(Self::take_staged(&target, slot)?),
      None => None,
    };
    let backup: Option<PathBuf> = target
      .exists()
      .then(|| StagedFile::name_sibling(&target, "replaced"))
      .transpose()?;

    self.files.push(XraySwappedFile { target, staged, backup });

    Ok(())
  }

  /// Swaps every staged file into place and deletes every marked one, undoing all of it when one fails.
  ///
  /// # Errors
  ///
  /// Returns the first failure, after putting back every file swapped before it.
  pub(crate) fn swap(mut self) -> XrfResult<()> {
    let Some(first) = self.files.first() else {
      return Ok(());
    };
    let journal: XrayDiskJournal = XrayDiskJournal::beside(&first.target)?;
    let entries: Vec<XrayJournalEntry> = self.files.iter().map(XraySwappedFile::to_entry).collect();

    journal.write(&entries, false)?;

    for index in 0..self.files.len() {
      if let Err(error) = self.files[index].swap() {
        XrayJournalEntry::roll_back(&entries);
        journal.remove();
        self.remove_directories();

        return Err(error);
      }
    }

    // Done from here on: a crash now rolls forward, keeping every swap.
    if let Err(error) = journal.write(&entries, true) {
      log::warn!("Could not mark a commit's journal done: {error}");
    }

    XrayJournalEntry::roll_forward(&entries);
    journal.remove();

    // Kept: they hold the files this commit created.
    self.directories.clear();

    Ok(())
  }

  /// The file a writer staged for `target`, taken out of its transaction so no other commit publishes it.
  fn take_staged(target: &Path, slot: &Mutex<Option<StagedFile>>) -> XrfResult<StagedFile> {
    // A panicked commit leaves the slot whole: it holds the staged file or nothing.
    let staged: StagedFile = slot
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .take()
      .ok_or_else(|| {
        XrfError::new_asset_error(format!(
          "cannot write '{}': its staged write was spent by an earlier commit",
          format_path(target)
        ))
      })?;

    if staged.get_target() != target {
      return Err(XrfError::new_asset_error(format!(
        "cannot write '{}': its staged write stands beside '{}'",
        format_path(target),
        format_path(staged.get_target())
      )));
    }

    // Swapped by one rename within the target's directory, and left over after a crash under a name the directory index
    // skips and the journal settles: anything else could cross volumes or be indexed as an asset.
    let is_sibling: bool = staged.get_staged_path().parent() == target.parent()
      && staged.get_staged_path().file_name().is_some_and(StagedFile::is_sibling);

    if !is_sibling {
      return Err(XrfError::new_asset_error(format!(
        "cannot write '{}': its staged write '{}' is not a sibling named by StagedFile::name_sibling",
        format_path(target),
        format_path(staged.get_staged_path())
      )));
    }

    Ok(staged)
  }

  /// Creates the directories above `target` that are not there, remembering each for a failed commit to remove.
  fn create_directories_above(&mut self, target: &Path) -> XrfResult<()> {
    let missing: Vec<&Path> = target
      .ancestors()
      .skip(1)
      .take_while(|directory| !directory.as_os_str().is_empty() && !directory.exists())
      .collect();

    for directory in missing.into_iter().rev() {
      fs::create_dir(directory).map_err(|error| {
        XrfError::new_asset_error(format!("failed to create '{}': {error}", format_path(directory)))
      })?;

      self.directories.push(directory.to_path_buf());
    }

    Ok(())
  }

  /// Removes the directories made for new files, deepest first, where nothing else came to sit in them.
  fn remove_directories(&mut self) {
    for directory in self.directories.drain(..).rev() {
      let _ = fs::remove_dir(&directory);
    }
  }
}

impl Drop for XrayDiskSwap {
  /// Removes what a commit that did not finish left: the staged files go with their handles, the directories made for
  /// them here.
  fn drop(&mut self) {
    self.files.clear();
    self.remove_directories();
  }
}
