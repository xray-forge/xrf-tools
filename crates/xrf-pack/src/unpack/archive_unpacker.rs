use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::ErrorKind::AlreadyExists;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;
use std::time::Duration;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rayon::{ThreadPool, ThreadPoolBuilder};
use xrf_archive::ArchiveFileDescriptor;
use xrf_archive::ArchiveProject;
use xrf_archive::write_descriptor_contents;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::path::{relative_to_prefix, to_host_relative};
use crate::unpack::archive_extract_result::{ArchiveExtractDirectoryResult, ArchiveExtractResult};
use crate::unpack::archive_unpack_progress::ArchiveUnpackProgress;
use crate::unpack::archive_unpack_result::ArchiveUnpackResult;

/// Writes the contents of an archive project back out to a directory.
pub struct ArchiveUnpacker;

impl ArchiveUnpacker {
  /// Workers an unpack run uses when its caller states no preference.
  ///
  /// The host's own parallelism rather than a fixed number, because the count sizes a real thread pool: a value chosen
  /// for one machine is idle capacity on a larger one and oversubscription on a smaller one. A host that cannot report
  /// its parallelism unpacks on a single worker, which is slower but never wrong.
  pub fn get_default_concurrency() -> NonZeroUsize {
    available_parallelism().unwrap_or(NonZeroUsize::MIN)
  }

  /// Write every file in the project beneath a destination root, up to `concurrency` entries at a time.
  ///
  /// Synchronous, and it owns the pool it runs on: nothing below this performs asynchronous I/O, so a caller on an
  /// executor has to move the whole call to a blocking thread rather than expect it to yield. One worker is a
  /// sequential run, so this is the only unpack entry point.
  ///
  /// The count is non-zero because `ThreadPoolBuilder::num_threads(0)` means "decide for me" to Rayon: a zero would
  /// quietly become one worker per core, which is the opposite of what a caller asking for a bound wants.
  ///
  /// A failure ends the run: no further entry is started, entries already started run to completion, and one of the
  /// errors is returned. Which one is unspecified, because entries are dispatched out of a hash table and finish out
  /// of order. Whatever was written stays on disk and the run is still reported as a failure — a partial tree reported
  /// as a success is worse, because nothing downstream can tell that the missing files were never written.
  pub fn unpack<P: AsRef<Path>>(
    project: &ArchiveProject,
    destination: P,
    concurrency: NonZeroUsize,
  ) -> XrfResult<ArchiveUnpackResult> {
    let destination: &Path = destination.as_ref();
    let mut progress: ArchiveUnpackProgress = ArchiveUnpackProgress::begin(project.files.len());

    Self::unpack_dirs(project, destination)?;
    progress.record_prepared();

    Self::build_pool(concurrency)?.install(|| {
      project.files.par_iter().try_for_each(|(_, descriptor)| -> XrfResult {
        // A directory row carries no payload, and the tree it names was created during preparation. It is counted
        // anyway, so the progress total stays the entry count the project reports holding.
        if !descriptor.is_directory {
          Self::unpack_file(destination, descriptor)?;
        }

        progress.record_unpacked();

        Ok(())
      })
    })?;

    Ok(Self::describe(
      project,
      destination,
      progress.get_prepared_at(),
      progress.elapsed(),
    ))
  }

  /// Write every archived file under one directory to a destination root.
  ///
  /// Keeps the layout below the prefix but not the prefix itself: extracting `configs\gameplay` into
  /// `C:\out` produces `C:\out\dialogs.xml`, not `C:\out\configs\gameplay\dialogs.xml`. The user picked
  /// the destination for the directory they named, so repeating that directory inside it is surprising.
  ///
  /// An empty prefix means the whole archive, which is what selecting the tree root does.
  pub fn extract_directory<P: AsRef<Path>>(
    project: &ArchiveProject,
    prefix: &str,
    destination: P,
  ) -> XrfResult<ArchiveExtractDirectoryResult> {
    let normalized: String = prefix.trim_end_matches(['\\', '/']).to_string();

    let mut extracted_count: usize = 0;
    let mut found: bool = false;
    let mut size: u64 = 0;

    for descriptor in project.files.values() {
      let Some(relative) = relative_to_prefix(&descriptor.name, &normalized) else {
        continue;
      };

      let target_path: PathBuf = destination.as_ref().join(to_host_relative(relative)?);

      found = true;

      if descriptor.is_directory {
        fs::create_dir_all(&target_path)?;

        continue;
      }

      if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
      }

      write_descriptor_contents(&mut Self::create_target(&target_path)?, descriptor)?;

      extracted_count += 1;
      size += descriptor.size_real as u64;
    }

    if !found {
      return Err(XrfError::new_not_found_error(format!(
        "Cannot extract '{normalized}' - no files in the archive are under it."
      )));
    }

    Ok(ArchiveExtractDirectoryResult {
      prefix: normalized,
      destination: format_path(destination.as_ref()).to_string(),
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

  fn describe(
    project: &ArchiveProject,
    destination: &Path,
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
      prepare_duration: prepared_at,
      unpack_duration: unpacked_at.saturating_sub(prepared_at),
      unpacked_size: project.get_real_size(),
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

  fn unpack_file<P: AsRef<Path>>(destination: P, descriptor: &ArchiveFileDescriptor) -> XrfResult {
    write_descriptor_contents(
      &mut Self::create_target(&Self::build_target_path(destination.as_ref(), descriptor)?)?,
      descriptor,
    )
  }

  fn unpack_dirs<P: AsRef<Path>>(project: &ArchiveProject, destination: P) -> XrfResult {
    let mut set: HashSet<PathBuf> = HashSet::new();

    for descriptor in project.files.values() {
      let target: PathBuf = Self::build_target_path(destination.as_ref(), descriptor)?;

      let directory: Option<PathBuf> = if descriptor.is_directory {
        Some(target)
      } else {
        target.parent().map(Into::into)
      };

      if let Some(directory) = directory {
        set.insert(directory);
      }
    }

    for path in set {
      match fs::create_dir_all(path) {
        Ok(_) => {}
        Err(error) if error.kind() == AlreadyExists => {}
        Err(error) => return Err(error.into()),
      }
    }

    Ok(())
  }

  /// Where one archived entry lands below a destination root.
  ///
  /// Both halves are engine paths, so both are crossed into host components rather than pushed whole: an entry named
  /// `configs\system.ltx` is a single component to `std::path` on Linux, which unpacks the tree as a flat directory of
  /// files with backslashes in their names.
  fn build_target_path(destination: &Path, descriptor: &ArchiveFileDescriptor) -> XrfResult<PathBuf> {
    let mut path: PathBuf = destination.into();

    path.push(to_host_relative(&descriptor.destination.to_string_lossy())?);
    path.push(to_host_relative(&descriptor.name)?);

    Ok(path)
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use xrf_archive::ArchiveFileDescriptor;

  use super::ArchiveUnpacker;

  #[test]
  fn an_entry_lands_under_its_volumes_root_as_a_real_tree() {
    let descriptor: ArchiveFileDescriptor = ArchiveFileDescriptor::new(0, String::from("configs\\system.ltx"), 0, 0, 0)
      .with_archive_paths(Path::new("textures.db0"), Path::new("gamedata\\"));

    assert_eq!(
      ArchiveUnpacker::build_target_path(Path::new("out"), &descriptor).expect("safe archive path"),
      PathBuf::from("out").join("gamedata").join("configs").join("system.ltx")
    );
  }

  /// The default is handed straight to a pool builder, so it has to be a usable worker count without further checking.
  /// A host that cannot report its own parallelism still has to unpack rather than fail or stall on zero workers.
  #[test]
  fn the_default_concurrency_is_always_a_usable_worker_count() {
    assert!(ArchiveUnpacker::get_default_concurrency().get() >= 1);
  }
}
