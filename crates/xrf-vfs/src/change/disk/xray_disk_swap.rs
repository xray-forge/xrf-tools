use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::change::disk::{XrayDiskJournal, XrayJournalEntry, XraySwappedFile};

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
  /// Stages `bytes` beside `target`, creating the directories above it, or, for `None`, marks `target` to be deleted.
  ///
  /// # Errors
  ///
  /// Returns an error when a directory or the staged file cannot be created, written or synced. What was staged
  /// before, and the directories made for it, are removed when the swap is dropped.
  pub(crate) fn stage(&mut self, target: PathBuf, bytes: Option<&[u8]>) -> XrfResult<()> {
    let staged: Option<StagedFile> = match bytes {
      Some(bytes) => {
        self.create_directories_above(&target)?;

        Some(
          StagedFile::stage(&target, &StagedFile::to_sibling(&target, "staged")?, bytes).map_err(|error| {
            XrfError::new_asset_error(format!("failed to stage '{}': {error}", format_path(&target)))
          })?,
        )
      }
      None => None,
    };
    let backup: Option<PathBuf> = target
      .exists()
      .then(|| StagedFile::to_sibling(&target, "replaced"))
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
