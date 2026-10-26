use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// One file as a commit's journal names it: its target, the staged content beside it and the kept file it replaced.
pub(crate) struct XrayJournalEntry {
  pub(crate) target: PathBuf,
  pub(crate) staged: Option<PathBuf>,
  pub(crate) backup: Option<PathBuf>,
}

impl XrayJournalEntry {
  /// The journal line naming the entry: target, staged and kept file, tab-separated, `-` for none.
  ///
  /// # Errors
  ///
  /// Returns an error when a path is not Unicode, which a journal line cannot hold.
  pub(crate) fn to_line(&self) -> XrfResult<String> {
    let fields: [Option<&Path>; 3] = [Some(&self.target), self.staged.as_deref(), self.backup.as_deref()];
    let mut line: String = String::new();

    for (index, field) in fields.into_iter().enumerate() {
      let field: &str = match field {
        None => "-",
        Some(path) => path.to_str().ok_or_else(|| {
          XrfError::new_asset_error(format!(
            "cannot commit '{}': its path is not Unicode",
            format_path(path)
          ))
        })?,
      };

      line.push_str(if index == 0 { "" } else { "\t" });
      line.push_str(field);
    }

    Ok(line)
  }

  /// The entry a journal line names, none where the line names no target.
  pub(crate) fn from_line(line: &str) -> Option<Self> {
    let mut fields = line
      .split('\t')
      .map(|field| (field != "-").then(|| PathBuf::from(field)));

    Some(Self {
      target: fields.next()??,
      staged: fields.next()?,
      backup: fields.next()?,
    })
  }

  /// Puts every file back as it was before the commit, as far as the swap got: a kept file over its target, a created
  /// one removed, and every staged file gone.
  pub(crate) fn roll_back(entries: &[Self]) {
    for entry in entries.iter().rev() {
      match &entry.backup {
        Some(backup) if backup.exists() => {
          if let Err(error) = fs::rename(backup, &entry.target) {
            log::error!(
              "Could not restore {} from {}: {error}",
              format_path(&entry.target),
              format_path(backup)
            );
          }
        }
        // Kept nothing: the target was not touched.
        Some(_) => {}
        // Nothing was there before, so a target there now is the commit's, once its staged file has moved.
        None => {
          if entry.staged.as_ref().is_some_and(|staged| !staged.exists()) {
            let _ = fs::remove_file(&entry.target);
          }
        }
      }

      if let Some(staged) = &entry.staged {
        let _ = fs::remove_file(staged);
      }
    }
  }

  /// Removes what a done commit kept beside its files.
  pub(crate) fn roll_forward(entries: &[Self]) {
    for path in entries
      .iter()
      .flat_map(|entry| entry.backup.iter().chain(&entry.staged))
    {
      if path.exists()
        && let Err(error) = fs::remove_file(path)
      {
        log::warn!("Could not remove {} after a commit: {error}", format_path(path));
      }
    }
  }
}
