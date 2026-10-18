use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome, JobScope};

use crate::unpack::archive_extract_options::{EXTRACT_PHASE_PREPARE, EXTRACT_PHASE_WRITE};
use crate::unpack::rooted_destination::RootedDestination;
use crate::unpack::tree_entry::TreeEntry;
use crate::unpack::tree_write_summary::TreeWriteSummary;

/// Writes a selected set of entries below a destination root, as wide as the installed pool.
pub(crate) struct TreeWriter;

impl TreeWriter {
  /// Creates every directory the entries need, then writes each file's payload through `write`.
  ///
  /// # Errors
  ///
  /// Returns whatever creating a directory or file, or `write`, answers with.
  pub(crate) fn write<T, F>(
    destination: &RootedDestination,
    entries: &[TreeEntry<T>],
    job: &JobHandle,
    write: F,
  ) -> XrfResult<TreeWriteSummary>
  where
    T: Sync,
    F: Fn(&T, &mut File) -> XrfResult<u64> + Sync,
  {
    let prepared: HashMap<PathBuf, PathBuf> = {
      let preparing: JobScope = job.enter(EXTRACT_PHASE_PREPARE, None);

      Self::create_directories(destination, entries, &preparing)?
    };

    let prepared_at: Duration = job.elapsed();
    let files: AtomicUsize = AtomicUsize::new(0);
    let bytes: AtomicU64 = AtomicU64::new(0);

    {
      let writing: JobScope = job.enter(EXTRACT_PHASE_WRITE, Some(entries.len() as u64));

      let outcome: XrfResult = entries.par_iter().try_for_each(|entry: &TreeEntry<T>| -> XrfResult {
        // Before the write rather than after it: a payload already being written cannot be halved without leaving a
        // truncated file that is indistinguishable from a short one, so the only safe boundary is this one.
        job.check_cancelled()?;

        // A directory record carries no payload, and the tree it names was created during preparation. It is counted
        // anyway, so the progress total stays the entry count the caller selected.
        if !entry.is_directory {
          let written: u64 = write(
            &entry.source,
            &mut Self::create_file(destination, &prepared, &entry.relative)?,
          )?;

          files.fetch_add(1, Ordering::Relaxed);
          bytes.fetch_add(written, Ordering::Relaxed);
        }

        writing.advance();

        Ok(())
      });

      // The run's own cancellation is control flow, not a failure. Any other error is the caller's to see.
      if let Err(error) = outcome
        && !matches!(error, XrfError::Cancelled { .. })
      {
        return Err(error);
      }
    }

    Ok(TreeWriteSummary {
      files: files.into_inner(),
      bytes: bytes.into_inner(),
      outcome: if job.is_cancelled() {
        JobOutcome::Cancelled
      } else {
        JobOutcome::Completed
      },
      prepared_at,
    })
  }

  /// Opens one entry's file, reusing the parent directory the preparation phase already verified.
  fn create_file(
    destination: &RootedDestination,
    prepared: &HashMap<PathBuf, PathBuf>,
    relative: &Path,
  ) -> XrfResult<File> {
    let verified: Option<(&PathBuf, &OsStr)> = relative
      .parent()
      .and_then(|parent| prepared.get(parent))
      .zip(relative.file_name());

    match verified {
      Some((parent, name)) => destination.create_file_in(parent, name),
      None => destination.create_file(relative),
    }
  }

  /// Creates every directory the entries need, and returns where each one landed.
  fn create_directories<T>(
    destination: &RootedDestination,
    entries: &[TreeEntry<T>],
    preparing: &JobScope,
  ) -> XrfResult<HashMap<PathBuf, PathBuf>> {
    let mut set: HashSet<&Path> = HashSet::new();

    for entry in entries {
      let directory: Option<&Path> = if entry.is_directory {
        Some(&entry.relative)
      } else {
        entry.relative.parent()
      };

      if let Some(directory) = directory {
        set.insert(directory);
      }
    }

    let mut created: HashMap<PathBuf, PathBuf> = HashMap::with_capacity(set.len());

    for path in set {
      let host: PathBuf = destination.create_directory(path)?;

      created.insert(path.to_path_buf(), host);
      preparing.advance();
    }

    Ok(created)
  }
}
