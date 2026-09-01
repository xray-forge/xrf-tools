use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::{ThreadPool, ThreadPoolBuilder};
use xrf_archive::ArchiveFileDescriptor;
use xrf_archive::ArchiveProject;
use xrf_archive::write_descriptor_contents;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobOutcome, JobScope};
use xrf_utils::format_path;

use crate::path::{relative_to_prefix, to_host_relative};
use crate::unpack::archive_extract_options::{ArchiveExtractOptions, EXTRACT_PHASE_WRITE};
use crate::unpack::archive_extract_result::{ArchiveExtractDirectoryResult, ArchiveExtractResult};
use crate::unpack::archive_unpack_options::{ArchiveUnpackOptions, UNPACK_PHASE_PREPARE, UNPACK_PHASE_WRITE};
use crate::unpack::archive_unpack_result::ArchiveUnpackResult;
use crate::unpack::rooted_destination::RootedDestination;

/// What one unpack run wrote, counted as it went.
///
/// Separate from the job's own counters because a job counts units for whoever is watching, while these are what the
/// result reports: a watcher may not exist, and the numbers still have to be true.
#[derive(Default)]
struct UnpackTally {
  files: AtomicUsize,
  bytes: AtomicU64,
}

/// Writes the contents of an archive project back out to a directory.
pub struct ArchiveUnpacker;

impl ArchiveUnpacker {
  /// Workers an unpack run uses when its caller states no preference.
  pub fn get_default_concurrency() -> NonZeroUsize {
    ArchiveUnpackOptions::get_default_concurrency()
  }

  /// Write every file in the project beneath a destination root, using the host's own parallelism.
  ///
  /// The plain door. A caller that wants to bound the run, watch it, or be able to stop it uses [`Self::unpack_opt`].
  pub fn unpack<P: AsRef<Path>>(project: &ArchiveProject, destination: P) -> XrfResult<ArchiveUnpackResult> {
    Self::unpack_opt(project, destination, ArchiveUnpackOptions::default())
  }

  /// Write every file in the project beneath a destination root, as `options` asks.
  ///
  /// Synchronous, and it owns the pool it runs on: nothing below this performs asynchronous I/O, so a caller on an
  /// executor has to move the whole call to a blocking thread rather than expect it to yield. One worker is a
  /// sequential run.
  ///
  /// The worker count is non-zero because `ThreadPoolBuilder::num_threads(0)` means "decide for me" to Rayon: a zero
  /// would quietly become one worker per core, which is the opposite of what a caller asking for a bound wants.
  ///
  /// Names come from the archive, so every write goes through a [`RootedDestination`] and lands below `destination`
  /// even where that tree already exists and holds links.
  ///
  /// A failure ends the run: no further entry is started, entries already started run to completion, and one of the
  /// errors is returned. Which one is unspecified, because entries are dispatched out of a hash table and finish out
  /// of order. Whatever was written stays on disk and the run is still reported as a failure — a partial tree reported
  /// as a success is worse, because nothing downstream can tell that the missing files were never written.
  ///
  /// Cancellation is the same shape with a different name: the run stops between entries, keeps what it wrote, and
  /// answers `Ok` carrying counts of what actually landed. It is not a failure, so it does not lose the description
  /// of the tree it left behind — which is the only way a caller can tell the user what to clean up.
  pub fn unpack_opt<P: AsRef<Path>>(
    project: &ArchiveProject,
    destination: P,
    options: ArchiveUnpackOptions,
  ) -> XrfResult<ArchiveUnpackResult> {
    let destination: RootedDestination = RootedDestination::new(destination.as_ref());
    let job = &options.job;
    let tally: UnpackTally = UnpackTally::default();

    destination.create_root()?;

    let prepared: HashMap<PathBuf, PathBuf> = {
      let preparing: JobScope = job.enter(UNPACK_PHASE_PREPARE, None);

      Self::unpack_dirs(project, &destination, &preparing)?
    };

    let prepared_at: Duration = job.elapsed();

    {
      let writing: JobScope = job.enter(UNPACK_PHASE_WRITE, Some(project.files.len() as u64));

      let outcome: XrfResult = Self::build_pool(options.concurrency)?.install(|| {
        project.files.par_iter().try_for_each(|(_, descriptor)| -> XrfResult {
          // Before the write rather than after it: a payload already being written cannot be halved without leaving a
          // truncated file that is indistinguishable from a short one, so the only safe boundary is this one.
          job.check_cancelled()?;

          // A directory row carries no payload, and the tree it names was created during preparation. It is counted
          // anyway, so the progress total stays the entry count the project reports holding.
          if !descriptor.is_directory {
            Self::unpack_file(&destination, &prepared, descriptor)?;

            tally.files.fetch_add(1, Ordering::Relaxed);
            tally
              .bytes
              .fetch_add(u64::from(descriptor.size_real), Ordering::Relaxed);
          }

          writing.advance();

          Ok(())
        })
      });

      // The run's own cancellation is control flow, not a failure. Any other error is the caller's to see.
      if let Err(error) = outcome
        && !matches!(error, XrfError::Cancelled { .. })
      {
        return Err(error);
      }
    }

    Ok(Self::describe_result(
      project,
      destination.get_root(),
      &tally,
      if job.is_cancelled() {
        JobOutcome::Cancelled
      } else {
        JobOutcome::Completed
      },
      prepared_at,
      job.elapsed(),
    ))
  }

  /// Write every archived file under one directory to a destination root.
  ///
  /// Keeps the layout below the prefix but not the prefix itself: extracting `configs\gameplay` into
  /// `C:\out` produces `C:\out\dialogs.xml`, not `C:\out\configs\gameplay\dialogs.xml`. The user picked
  /// the destination for the directory they named, so repeating that directory inside it is surprising.
  ///
  /// An empty prefix means the whole archive, which is what selecting the tree root does. What lies below the prefix
  /// is archive-controlled, so it is written through a [`RootedDestination`] rather than joined and opened.
  pub fn extract_directory<P: AsRef<Path>>(
    project: &ArchiveProject,
    prefix: &str,
    destination: P,
  ) -> XrfResult<ArchiveExtractDirectoryResult> {
    Self::extract_directory_opt(project, prefix, destination, ArchiveExtractOptions::default())
  }

  /// Write every archived file under one directory to a destination root, as `options` asks.
  ///
  /// Keeps the layout below the prefix but not the prefix itself: extracting `configs\gameplay` into
  /// `C:\out` produces `C:\out\dialogs.xml`, not `C:\out\configs\gameplay\dialogs.xml`. The user picked
  /// the destination for the directory they named, so repeating that directory inside it is surprising.
  ///
  /// An empty prefix means the whole archive, which is what selecting the tree root does. What lies below the prefix
  /// is archive-controlled, so it is written through a [`RootedDestination`] rather than joined and opened.
  ///
  /// Cancellation lands between entries and keeps what it wrote, the same way an unpack does: the destination may hold
  /// the caller's own files, and nothing here can tell those from this run's.
  pub fn extract_directory_opt<P: AsRef<Path>>(
    project: &ArchiveProject,
    prefix: &str,
    destination: P,
    options: ArchiveExtractOptions,
  ) -> XrfResult<ArchiveExtractDirectoryResult> {
    let normalized: String = prefix.trim_end_matches(['\\', '/']).to_string();
    let destination: RootedDestination = RootedDestination::new(destination.as_ref());
    let job = &options.job;

    destination.create_root()?;

    // Selected before anything is written, so the run knows how much it is about to do. The alternative - filtering
    // inside the write loop - leaves the total unknowable until the end, which is exactly when it stops being useful.
    let selected: Vec<(&ArchiveFileDescriptor, PathBuf)> = project
      .files
      .values()
      .filter_map(|descriptor| relative_to_prefix(&descriptor.name, &normalized).map(|relative| (descriptor, relative)))
      .map(|(descriptor, relative)| to_host_relative(relative).map(|relative| (descriptor, relative)))
      .collect::<XrfResult<Vec<(&ArchiveFileDescriptor, PathBuf)>>>()?;

    if selected.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "Cannot extract '{normalized}' - no files in the archive are under it."
      )));
    }

    let mut extracted_count: usize = 0;
    let mut size: u64 = 0;
    let mut outcome: JobOutcome = JobOutcome::Completed;

    let extracting: JobScope = job.enter(EXTRACT_PHASE_WRITE, Some(selected.len() as u64));

    for (descriptor, relative) in &selected {
      // Before the write rather than after it: a payload already being written cannot be halved without leaving a
      // truncated file indistinguishable from a short one.
      if job.is_cancelled() {
        outcome = JobOutcome::Cancelled;

        break;
      }

      if descriptor.is_directory {
        destination.create_directory(relative)?;
      } else {
        write_descriptor_contents(&mut destination.create_file(relative)?, descriptor)?;

        extracted_count += 1;
        size += descriptor.size_real as u64;
      }

      extracting.advance();
    }

    Ok(ArchiveExtractDirectoryResult {
      prefix: normalized,
      destination: format_path(destination.get_root()).to_string(),
      outcome,
      extracted_count,
      size,
    })
  }

  /// Write one archived file to an exact path of the caller's choosing.
  pub fn extract_file<P: AsRef<Path>>(
    project: &ArchiveProject,
    name: &str,
    destination: P,
  ) -> XrfResult<ArchiveExtractResult> {
    let descriptor: &ArchiveFileDescriptor = project.files.get(name).ok_or_else(|| {
      XrfError::new_not_found_error(format!("Cannot extract '{name}' - no such file in the archive."))
    })?;

    if descriptor.is_directory {
      return Err(XrfError::new_invalid_error(format!(
        "Cannot extract '{}' as a file - it is a directory record.",
        descriptor.name
      )));
    }

    if let Some(parent) = destination.as_ref().parent() {
      fs::create_dir_all(parent)?;
    }

    write_descriptor_contents(&mut Self::create_target(destination.as_ref())?, descriptor)?;

    Ok(ArchiveExtractResult {
      name: descriptor.name.clone(),
      destination: format_path(destination.as_ref()).to_string(),
      size: descriptor.size_real as u64,
    })
  }

  /// A pool built for one run, never the process-wide one.
  ///
  /// Installing onto the global pool would make one command's chosen worker count everything else's too, and the
  /// caller asked to bound this unpack rather than the process it happens to run in.
  fn build_pool(concurrency: NonZeroUsize) -> XrfResult<ThreadPool> {
    ThreadPoolBuilder::new()
      .num_threads(concurrency.get())
      .thread_name(|index| format!("xrf-unpack-{index}"))
      .build()
      .map_err(|error| XrfError::new_unexpected_error(format!("cannot start {concurrency} unpack worker(s): {error}")))
  }

  fn describe_result(
    project: &ArchiveProject,
    destination: &Path,
    tally: &UnpackTally,
    outcome: JobOutcome,
    prepared_at: Duration,
    unpacked_at: Duration,
  ) -> ArchiveUnpackResult {
    ArchiveUnpackResult {
      archives: project
        .archives
        .iter()
        .map(|it| format_path(&it.path).to_string())
        .collect(),
      destination: format_path(destination).to_string(),
      duration: unpacked_at,
      outcome,
      files_total: project.files.len(),
      files_unpacked: tally.files.load(Ordering::Relaxed),
      prepare_duration: prepared_at,
      unpack_duration: unpacked_at.saturating_sub(prepared_at),
      unpacked_size: tally.bytes.load(Ordering::Relaxed),
    }
  }

  fn create_target(path: &Path) -> XrfResult<File> {
    Ok(
      File::options()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?,
    )
  }

  /// Writes one entry, reusing the parent directory the preparation phase already verified.
  ///
  /// `prepared` holds every directory the entries named, so the lookup normally hits and the walk down is skipped. The
  /// miss path is kept because it costs nothing and is the only thing standing between a future change in what
  /// preparation collects and an entry that never reaches disk.
  fn unpack_file(
    destination: &RootedDestination,
    prepared: &HashMap<PathBuf, PathBuf>,
    descriptor: &ArchiveFileDescriptor,
  ) -> XrfResult {
    let relative: PathBuf = Self::build_relative_path(descriptor)?;
    let verified: Option<(&PathBuf, &OsStr)> = relative
      .parent()
      .and_then(|parent| prepared.get(parent))
      .zip(relative.file_name());

    let mut target: File = match verified {
      Some((parent, name)) => destination.create_file_in(parent, name)?,
      None => destination.create_file(&relative)?,
    };

    write_descriptor_contents(&mut target, descriptor)
  }

  /// Creates every directory the entries need, and returns where each one landed.
  ///
  /// The map is what stops the write phase walking the same components again for every file in a directory: it holds
  /// one verified host path per relative directory, keyed the way [`Self::build_relative_path`] spells it.
  fn unpack_dirs(
    project: &ArchiveProject,
    destination: &RootedDestination,
    preparing: &JobScope,
  ) -> XrfResult<HashMap<PathBuf, PathBuf>> {
    let mut set: HashSet<PathBuf> = HashSet::new();

    for descriptor in project.files.values() {
      let target: PathBuf = Self::build_relative_path(descriptor)?;

      let directory: Option<PathBuf> = if descriptor.is_directory {
        Some(target)
      } else {
        target.parent().map(Into::into)
      };

      if let Some(directory) = directory {
        set.insert(directory);
      }
    }

    let mut created: HashMap<PathBuf, PathBuf> = HashMap::with_capacity(set.len());

    for path in set {
      let host: PathBuf = destination.create_directory(&path)?;

      created.insert(path, host);
      preparing.advance();
    }

    Ok(created)
  }

  /// Where one archived entry lands, relative to a destination root.
  ///
  /// Both halves are engine paths, so both are crossed into host components rather than pushed whole: an entry named
  /// `configs\system.ltx` is a single component to `std::path` on Linux, which unpacks the tree as a flat directory of
  /// files with backslashes in their names.
  ///
  /// Relative rather than already joined, because the destination is only ever reached one component at a time.
  fn build_relative_path(descriptor: &ArchiveFileDescriptor) -> XrfResult<PathBuf> {
    let mut path: PathBuf = to_host_relative(&descriptor.destination.to_string_lossy())?;

    path.push(to_host_relative(&descriptor.name)?);

    Ok(path)
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};
  use std::sync::Arc;

  use xrf_archive::ArchiveFileDescriptor;

  use super::ArchiveUnpacker;

  #[test]
  fn an_entry_lands_under_its_volumes_root_as_a_real_tree() {
    let descriptor: ArchiveFileDescriptor = ArchiveFileDescriptor::new(0, String::from("configs\\system.ltx"), 0, 0, 0)
      .with_archive_paths(
        &Arc::from(Path::new("textures.db0")),
        &Arc::from(Path::new("gamedata\\")),
      );

    assert_eq!(
      ArchiveUnpacker::build_relative_path(&descriptor).expect("safe archive path"),
      PathBuf::from("gamedata").join("configs").join("system.ltx")
    );
  }

  /// The default is handed straight to a pool builder, so it has to be a usable worker count without further checking.
  /// A host that cannot report its own parallelism still has to unpack rather than fail or stall on zero workers.
  #[test]
  fn the_default_concurrency_is_always_a_usable_worker_count() {
    assert!(ArchiveUnpacker::get_default_concurrency().get() >= 1);
  }
}
