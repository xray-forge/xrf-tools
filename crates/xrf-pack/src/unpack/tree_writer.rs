use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobOutcome, JobScope};
use xrf_utils::{HostDestination, HostRelativePath};

use crate::unpack::archive_extract_options::{EXTRACT_PHASE_PREPARE, EXTRACT_PHASE_WRITE};
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
    destination: &HostDestination,
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
    let written_by: HashMap<String, usize> = Self::find_writers(entries);
    let files: AtomicUsize = AtomicUsize::new(0);
    let bytes: AtomicU64 = AtomicU64::new(0);

    let is_cancelled: bool = {
      let writing: JobScope = job.enter(EXTRACT_PHASE_WRITE, Some(entries.len() as u64));

      let outcome: XrfResult = entries
        .par_iter()
        .enumerate()
        .try_for_each(|(index, entry)| -> XrfResult {
          // Before the write rather than after it: a payload already being written cannot be halved without leaving a
          // truncated file that is indistinguishable from a short one, so the only safe boundary is this one.
          job.check_cancelled()?;

          // A directory record carries no payload, and the tree it names was created during preparation; an entry the
          // host stores in the same file as a later one is passed over. Both are counted anyway, so the progress total
          // stays the entry count the caller selected.
          if !entry.is_directory && written_by.get(&Self::to_host_key(&entry.relative)) == Some(&index) {
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

      // The run's own cancellation is control flow, not a failure, and counts only where it stopped the run. Any other
      // error is the caller's to see.
      match outcome {
        Ok(()) => false,
        Err(XrfError::Cancelled { .. }) => true,
        Err(error) => return Err(error),
      }
    };

    Ok(TreeWriteSummary {
      files: files.into_inner(),
      bytes: bytes.into_inner(),
      outcome: if is_cancelled {
        JobOutcome::Cancelled
      } else {
        JobOutcome::Completed
      },
      prepared_at,
    })
  }

  /// Which entry writes each file the host keeps: the last of the entries it stores in one file, so two workers never
  /// write the same file at once.
  fn find_writers<T>(entries: &[TreeEntry<T>]) -> HashMap<String, usize> {
    entries
      .iter()
      .enumerate()
      .filter(|(_, entry)| !entry.is_directory)
      .map(|(index, entry)| (Self::to_host_key(&entry.relative), index))
      .collect()
  }

  /// The file `relative` lands in as the host tells files apart: Windows names a file regardless of case.
  fn to_host_key(relative: &HostRelativePath) -> String {
    let path: String = relative.as_path().to_string_lossy().into_owned();

    if cfg!(windows) { path.to_lowercase() } else { path }
  }

  /// Opens one entry's file, reusing the parent directory the preparation phase already verified.
  fn create_file(
    destination: &HostDestination,
    prepared: &HashMap<PathBuf, PathBuf>,
    relative: &HostRelativePath,
  ) -> XrfResult<File> {
    let verified: Option<(&PathBuf, &OsStr)> = relative
      .as_path()
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
    destination: &HostDestination,
    entries: &[TreeEntry<T>],
    preparing: &JobScope,
  ) -> XrfResult<HashMap<PathBuf, PathBuf>> {
    let mut set: HashSet<HostRelativePath> = HashSet::new();

    for entry in entries {
      let directory: Option<HostRelativePath> = if entry.is_directory {
        Some(entry.relative.clone())
      } else {
        entry.relative.parent()
      };

      if let Some(directory) = directory {
        set.insert(directory);
      }
    }

    let mut created: HashMap<PathBuf, PathBuf> = HashMap::with_capacity(set.len());

    for path in set {
      let host: PathBuf = destination.create_directory(&path)?;

      created.insert(path.as_path().to_path_buf(), host);
      preparing.advance();
    }

    Ok(created)
  }
}
