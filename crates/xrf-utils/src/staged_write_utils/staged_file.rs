use std::fs::{self, File, OpenOptions};
use std::io::{Result as IoResult, Write};
use std::path::{Path, PathBuf};

#[cfg(feature = "staging-faults")]
use crate::staged_write_utils::staging_faults;
use crate::staged_write_utils::sync_parent_directory;

/// A file's next content written and synced in a sibling of it, not yet in its place: the first half of a staged
/// write, for a caller that publishes several files together. Removed when dropped unpublished.
#[derive(Debug)]
pub struct StagedFile {
  target: PathBuf,
  staged: PathBuf,
  is_published: bool,
}

impl StagedFile {
  /// Writes and syncs `contents` at `staged`, a name from [`to_staged_sibling`](crate::to_staged_sibling), to replace
  /// `target` once published;
  /// a replacement inherits `target`'s permissions.
  ///
  /// # Errors
  ///
  /// Returns an IO error, with nothing left at `staged`, when it cannot be created, written or synced.
  pub fn stage(target: &Path, staged: &Path, contents: &[u8]) -> IoResult<Self> {
    let result: IoResult<()> = (|| -> IoResult<()> {
      let mut staged_file: File = OpenOptions::new().write(true).create_new(true).open(staged)?;

      staged_file.write_all(contents)?;

      #[cfg(feature = "staging-faults")]
      if let Some(error) = staging_faults::take_injected_failure() {
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
    if let Some(error) = staging_faults::take_injected_publish_failure() {
      return Err(error);
    }

    // One rename on every platform: `fs::rename` replaces an existing destination everywhere Rust supports, Windows
    // included, through `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`.
    fs::rename(&self.staged, &self.target)?;

    self.is_published = true;

    sync_parent_directory(&self.target);

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
