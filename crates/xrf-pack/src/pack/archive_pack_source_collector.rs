//! The walk that decides what one packing run will write.
//!
//! Discovery only. Whether a name the walk reaches is wanted is `archive_pack_config_rules.rs`; how a host path
//! becomes an archive name is `crate::path`; which name the engine registers it under, and that it is registered once,
//! is `archive_pack_name_table.rs`.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, Error as WalkError, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobScope;
use xrf_utils::format_path;

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory};
use crate::pack::archive_pack_entry::ArchivePackEntry;
use crate::pack::archive_pack_name_table::ArchivePackNameTable;
use crate::pack::archive_pack_source::ArchivePackSource;
use crate::path::{normalize_archive_name, to_archive_name};

/// What the walk has found so far, as it found it.
///
/// The accumulator is the operation rather than a value beside it: nothing outside one walk has any use for a
/// half-discovered tree, and [`Self::collect_opt`] is the only way to reach one.
#[derive(Default)]
pub(crate) struct ArchivePackSourceCollector {
  files: Vec<ArchivePackEntry>,
  directories: Vec<String>,
  skipped: usize,
}

impl ArchivePackSourceCollector {
  /// Walk the configured roots and decide what goes into the archive, counting what it finds against `collecting`.
  ///
  /// Files come out in the order of their engine names, one per engine name. xrCompress emitted them in filesystem
  /// enumeration order; the engine indexes by name and does not care, and a stable order makes output reproducible.
  ///
  /// The total is unknowable here by definition — the walk is what discovers it — so this reports a rising count with
  /// no denominator rather than inventing one. That is the honest state for a phase that can take minutes on a large
  /// source tree and is the reason it is worth reporting at all.
  ///
  /// # Errors
  ///
  /// Returns a discovery failure for a root or subtree that could not be enumerated, for a listed file that does not
  /// resolve, and whatever registration refuses.
  pub(crate) fn collect_opt(config: &ArchivePackConfig, collecting: &JobScope) -> XrfResult<ArchivePackSource> {
    let mut collector: Self = Self::default();

    collector.walk(config)?;
    collector.register(config, collecting)
  }

  /// Reach every root the configuration selects.
  fn walk(&mut self, config: &ArchivePackConfig) -> XrfResult<()> {
    // A config that names nothing packs the whole tree, which is what xrCompress does when handed a
    // directory and no LTX. Naming nothing is far more likely to mean "everything" than "an empty archive".
    if config.include_directories.is_empty() && config.include_files.is_empty() {
      self.collect_directory(
        config,
        &ArchivePackDirectory {
          path: String::new(),
          is_recursive: true,
        },
      )?;
    }

    for directory in &config.include_directories {
      self.collect_directory(config, directory)?;
    }

    for name in &config.include_files {
      self.collect_file(config, name)?;
    }

    Ok(())
  }

  /// Close the walk: fold what it found into engine names, and report the size of the set that survived.
  fn register(self, config: &ArchivePackConfig, collecting: &JobScope) -> XrfResult<ArchivePackSource> {
    let names: ArchivePackNameTable = ArchivePackNameTable::register(config, self.files, self.directories)?;

    // Counted once the set is known rather than per file found: registration folds duplicates, so a running count
    // would go down again, and a progress number that goes backwards is worse than one that arrives late.
    collecting.advance_by(names.get_files().len() as u64);

    Ok(ArchivePackSource {
      names,
      skipped: self.skipped,
    })
  }

  /// Take one file `[include_files]` names, whatever the directory rules would have said about it.
  ///
  /// Listed files bypass those rules exactly as they do in xrCompress, but a name that does not resolve is a
  /// configuration error rather than something to pass over, which is what this otherwise unused lookup
  /// establishes.
  fn collect_file(&mut self, config: &ArchivePackConfig, name: &str) -> XrfResult<()> {
    let name: String = normalize_archive_name(name);
    let path: PathBuf = config.source.join(name.replace('\\', "/"));

    path.metadata()?;

    self.files.push(ArchivePackEntry { name, path });

    Ok(())
  }

  fn collect_directory(&mut self, config: &ArchivePackConfig, directory: &ArchivePackDirectory) -> XrfResult<()> {
    let root: PathBuf = if directory.path.is_empty() {
      config.source.clone()
    } else {
      config.source.join(directory.path.replace('\\', "/"))
    };

    if !is_present_source_root(&root)? {
      return Ok(());
    }

    // A non-recursive include still names its immediate subdirectories, matching `FS_ListFolders` without
    // `FS_RootOnly`, so the archive lists them even when their contents stay out.
    let walk: WalkDir = if directory.is_recursive {
      WalkDir::new(&root)
    } else {
      WalkDir::new(&root).max_depth(1)
    };

    // Every failure the walk reports ends the run. A packed volume set is read as a complete build of what the
    // configuration selected, so a subtree that could not be enumerated has to be an error rather than an omission.
    for entry in walk
      .into_iter()
      .filter_entry(|entry| !is_pruned_directory(config, entry))
    {
      let entry: DirEntry = entry.map_err(|error| walk_error(&root, error))?;
      let path: &Path = entry.path();
      let name: String = to_archive_name(&config.source, path)?;

      if name.is_empty() || config.is_excluded_directory(&name) {
        continue;
      }

      if entry.file_type().is_dir() {
        self.directories.push(name);
      } else if entry.file_type().is_file() {
        if config.is_skipped_file(&name) {
          self.skipped += 1;

          continue;
        }

        self.files.push(ArchivePackEntry {
          name,
          path: path.into(),
        });
      }
    }

    Ok(())
  }
}

/// Whether an included root is there at all, refusing an answer the filesystem could not give.
///
/// A root that is not there contributes nothing, which is how a shared configuration names optional trees. Every
/// other failure is a discovery failure: `Path::exists` reports a directory it may not read as absent too, and
/// packing would then publish a volume set missing everything below it.
fn is_present_source_root(root: &Path) -> XrfResult<bool> {
  match fs::metadata(root) {
    Ok(_) => Ok(true),
    Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
    Err(error) => Err(XrfError::new_io_error(
      format!(
        "Failed to read packing source directory '{}': {error}",
        format_path(root)
      ),
      error.kind(),
    )),
  }
}

/// Turn a walk failure into an error naming what could not be read.
///
/// The walker knows which entry it tripped on for everything below the root, and names the root itself otherwise.
fn walk_error(root: &Path, error: WalkError) -> XrfError {
  let path: String = format_path(error.path().unwrap_or(root)).to_string();

  // The wrapper repeats the path it already named, so the failure itself is what carries here.
  match error.io_error() {
    Some(cause) => XrfError::new_io_error(format!("Failed to read packing source '{path}': {cause}"), cause.kind()),
    None => XrfError::new_invalid_error(format!("Failed to read packing source '{path}': {error}")),
  }
}

/// Whether a directory can be dropped without descending into it.
///
/// A recursive exclusion covers everything below the directory it names, so nothing inside can be selected and
/// reading it would only turn a intentionally excluded corner of the tree into a packing failure. A plain exclusion
/// drops the directory alone while its contents still pack, so it must not prune. A directory whose name cannot be
/// expressed is kept, so the walk reports it instead of losing it here.
fn is_pruned_directory(config: &ArchivePackConfig, entry: &DirEntry) -> bool {
  entry.file_type().is_dir()
    && to_archive_name(&config.source, entry.path())
      .is_ok_and(|name| !name.is_empty() && config.is_recursively_excluded_directory(&name))
}
