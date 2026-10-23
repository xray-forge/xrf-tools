use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path, sync_parent_directory, to_staged_sibling};

use crate::change::xray_journal_entry::XrayJournalEntry;

/// A commit's journal beside its first target, naming every file it swaps: a crash leaves it for the next mount of the
/// directory to settle, rolled back while the swap was under way and forward once every file was swapped.
pub(crate) struct XrayDiskJournal {
  path: PathBuf,
}

impl XrayDiskJournal {
  /// The first line while a commit's files are being swapped, which a recovery rolls back.
  const PENDING: &str = "xrf-commit pending";
  /// The first line once every file is swapped, which a recovery rolls forward by removing what was set aside.
  const DONE: &str = "xrf-commit done";

  /// The journal of a commit whose first target is `target`.
  ///
  /// # Errors
  ///
  /// Returns an error when `target` names no file a sibling can sit beside.
  pub(crate) fn beside(target: &Path) -> XrfResult<Self> {
    Ok(Self {
      path: to_staged_sibling(target, "commit")?,
    })
  }

  /// Writes and syncs the journal naming `entries`, as under way, or done.
  ///
  /// # Errors
  ///
  /// Returns an error when a path is not Unicode, or the journal cannot be staged or published.
  pub(crate) fn write(&self, entries: &[XrayJournalEntry], is_done: bool) -> XrfResult<()> {
    let mut text: String = format!("{}\n", if is_done { Self::DONE } else { Self::PENDING });

    for entry in entries {
      text.push_str(&entry.to_line()?);
      text.push('\n');
    }

    StagedFile::stage(&self.path, &to_staged_sibling(&self.path, "tmp")?, text.as_bytes())
      .and_then(StagedFile::publish)
      .map_err(|error| XrfError::new_asset_error(format!("failed to journal a commit: {error}")))
  }

  /// Removes the journal once what it names is settled.
  pub(crate) fn remove(&self) {
    if let Err(error) = fs::remove_file(&self.path) {
      log::warn!(
        "Could not remove the commit journal {}: {error}",
        format_path(&self.path)
      );
    }

    sync_parent_directory(&self.path);
  }

  /// Whether a file name is a commit journal's.
  pub(crate) fn is_journal(name: &str) -> bool {
    name.starts_with('.') && name.contains(".xrf-commit-")
  }

  /// Settles what a commit cut short left: rolled back where the journal at `path` says its swap was under way, rolled
  /// forward where every swap was done, then the journal removed. An unreadable journal is left for a person to look at.
  pub(crate) fn recover(path: &Path) {
    let journal: Self = Self {
      path: path.to_path_buf(),
    };
    let text: String = match fs::read_to_string(path) {
      Ok(text) => text,
      Err(error) => {
        log::warn!("Could not read the commit journal {}: {error}", format_path(path));

        return;
      }
    };
    let mut lines = text.lines();
    let state: Option<&str> = lines.next();
    let entries: Vec<XrayJournalEntry> = lines.filter_map(XrayJournalEntry::from_line).collect();

    match state {
      Some(Self::PENDING) => {
        log::warn!(
          "Rolling back a commit of {} file(s) cut short, journalled in {}",
          entries.len(),
          format_path(path)
        );

        XrayJournalEntry::roll_back(&entries);
      }
      Some(Self::DONE) => XrayJournalEntry::roll_forward(&entries),
      _ => {
        log::warn!("Leaving the unrecognised commit journal {}", format_path(path));

        return;
      }
    }

    journal.remove();
  }

  /// Writes the journal a commit cut short at `is_done` would leave at `path`, for a test standing in for a crash.
  #[cfg(test)]
  pub(crate) fn write_for_test(path: &Path, is_done: bool, files: &[(&Path, Option<&Path>, Option<&Path>)]) {
    let entries: Vec<XrayJournalEntry> = files
      .iter()
      .map(|(target, staged, backup)| XrayJournalEntry {
        target: target.to_path_buf(),
        staged: staged.map(Path::to_path_buf),
        backup: backup.map(Path::to_path_buf),
      })
      .collect();

    Self {
      path: path.to_path_buf(),
    }
    .write(&entries, is_done)
    .expect("journal written");
  }
}
