use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::Serialize;
use walkdir::WalkDir;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::archive_descriptor::ArchiveDescriptor;
use crate::archive_file_descriptor::ArchiveFileDescriptor;
use crate::project::archive_project_read_policy::ArchiveProjectReadPolicy;
use crate::reader::ArchiveReader;

/// One volume set at a path the caller names, merged into a single name table.
///
/// Scoped to a path on purpose: which directories of an installation hold volumes is a question the mount planner in
/// `xrf-vfs` answers (`XrayMountPlan::from_fsgame`), and answering it here too would put `fsgame.ltx` knowledge in the
/// volume-format layer and give the same declaration two readers.
///
/// Later volumes win the merge, so a patch volume shadows the entry it replaces.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProject {
  /// Volumes in merge order: a later one wins the name table, so a caller searching them as separate sources must
  /// search them in reverse to resolve an entry to the bytes this project's table names.
  pub archives: Vec<ArchiveDescriptor>,
  pub files: HashMap<String, ArchiveFileDescriptor>,
  pub read_policy: ArchiveProjectReadPolicy,
  /// The tightest path holding exactly these volumes: the volume itself when one file was read, the volumes' common
  /// parent when a directory was walked. Mounting it reaches this project's entries and no others, which is what a
  /// caller reading an entry's bytes back out of the filesystem needs.
  pub root: PathBuf,
  pub size_real: u64,
}

impl ArchiveProject {
  /// Reads one archive file or all archive volumes recursively under a directory.
  ///
  /// # Errors
  ///
  /// Returns an error when no archive volume is found or a volume cannot be read.
  pub fn new(path: impl AsRef<Path>) -> XrfResult<Self> {
    Self::read_to_depth(path.as_ref(), usize::MAX)
  }

  /// Reads one archive file or archive volumes directly under a directory.
  ///
  /// Use this for nonrecursive `fsgame.ltx` archive aliases; recursive discovery would include subdirectories planned as
  /// separate mounts.
  ///
  /// # Errors
  ///
  /// Returns an error when no archive volume is found or a volume cannot be read.
  pub fn new_shallow(path: impl AsRef<Path>) -> XrfResult<Self> {
    Self::read_to_depth(path.as_ref(), 1)
  }

  /// The volumes [`Self::new`] would read at a path, in the order it merges them.
  ///
  /// Published so a caller mounting the same path as sources of its own reads the identical volume set — the archives
  /// explorer lists a project and then mounts its root to preview an entry, and a shallower discovery there offers a
  /// file the preview cannot reach.
  pub fn discover_volumes(path: impl AsRef<Path>) -> Vec<PathBuf> {
    Self::discover_to_depth(path.as_ref(), usize::MAX)
  }

  /// Volume paths at a path, in merge order: a named file is itself, a directory is walked to `depth`.
  fn discover_to_depth(path: &Path, depth: usize) -> Vec<PathBuf> {
    if path.is_file() {
      return vec![path.to_path_buf()];
    }

    let mut volumes: Vec<PathBuf> = WalkDir::new(path)
      .max_depth(depth)
      .into_iter()
      .filter_map(Result::ok)
      .map(|entry| entry.into_path())
      .filter(|path| ArchiveDescriptor::is_valid_db_path(path))
      .collect();

    Self::sort_volumes(&mut volumes);

    volumes
  }

  fn read_to_depth(path: &Path, depth: usize) -> XrfResult<Self> {
    let mut files: HashMap<String, ArchiveFileDescriptor> = HashMap::new();
    let is_single_volume: bool = path.is_file();

    if !is_single_volume {
      log::info!("Reading archive directory: {}", format_path(path));
    }

    let volumes: Vec<PathBuf> = Self::discover_to_depth(path, depth);
    let mut archives: Vec<ArchiveDescriptor> = Vec::with_capacity(volumes.len());

    for volume in &volumes {
      log::info!("Reading archive file: {}", format_path(volume));

      archives.push(ArchiveReader::from_path(volume)?.read_archive()?);
    }

    if archives.is_empty() {
      return Err(XrfError::new_read_error(format!(
        "Unable to read archives at location {}",
        format_path(path)
      )));
    }

    for archive in &archives {
      for (name, descriptor) in &archive.files {
        files.insert(name.clone(), descriptor.clone());
      }
    }

    // A named volume is its own root. Its parent would be a wider path than the project, and mounting that to read an
    // entry back would answer out of whichever sibling volume shadows the name.
    let root: PathBuf = if is_single_volume {
      path.to_path_buf()
    } else {
      Self::root_from_volumes(&volumes)
    };
    let size_real: u64 = files.values().map(|file| u64::from(file.size_real)).sum();

    Ok(Self {
      archives,
      files,
      read_policy: ArchiveProjectReadPolicy::default(),
      root,
      size_real,
    })
  }

  /// Bytes the merged name table's entries occupy once unpacked.
  ///
  /// Summed over the merged table rather than over the volumes, so an entry a later volume overrides is counted once.
  pub fn get_real_size(&self) -> u64 {
    self.size_real
  }

  /// Bytes the merged name table's entries occupy as stored.
  pub fn get_compressed_size(&self) -> u64 {
    self.files.values().map(|file| u64::from(file.size_compressed)).sum()
  }

  /// Orders volumes the way the engine registers them, so the last one merged is the one it would answer with.
  ///
  /// `CLocatorAPI::Recurse` sorts a directory's entries by name and processes them in place, descending into a
  /// subdirectory as its name comes up (`xray-16/src/xrCore/LocatorAPI.cpp`, and identically in `xray-monolith`). That
  /// is component-wise path order, which is what `Path`'s own ordering already is.
  ///
  /// No volume is special. A `patches` directory used to be forced last here, which no engine does: precedence between
  /// declared archive directories is their `fsgame.ltx` declaration order, and Anomaly declares `$arch_dir_addons$`
  /// after `$arch_dir_patches$`. Open the installation rather than its `db` directory to get that order.
  fn sort_volumes(volumes: &mut [PathBuf]) {
    volumes.sort();
  }

  fn root_from_volumes(volumes: &[PathBuf]) -> PathBuf {
    let Some(first) = volumes.first() else {
      return PathBuf::new();
    };
    let mut common: Vec<OsString> = first
      .parent()
      .unwrap_or_else(|| Path::new(""))
      .components()
      .map(|component| component.as_os_str().to_owned())
      .collect();

    for volume in &volumes[1..] {
      let components: Vec<OsString> = volume
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .components()
        .map(|component| component.as_os_str().to_owned())
        .collect();
      let common_length: usize = common
        .iter()
        .zip(&components)
        .take_while(|(left, right)| left == right)
        .count();

      common.truncate(common_length);
    }

    common.into_iter().collect()
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use crate::archive_descriptor::ArchiveDescriptor;

  use super::ArchiveProject;

  #[test]
  fn volumes_merge_in_the_order_the_engine_registers_them() {
    // `Recurse` sorts each directory's entries by name and descends as the name comes up, so a subdirectory is read
    // where its own name sorts rather than before or after every file. `patches` is not special to any engine.
    let mut volumes: [PathBuf; 5] = [
      PathBuf::from("/game/db/patches/xpatch_1.db"),
      PathBuf::from("/game/db/textures/textures.db0"),
      PathBuf::from("/game/db/configs.db0"),
      PathBuf::from("/game/db/textures.db0"),
      PathBuf::from("/game/db/addons/mod.db0"),
    ];

    ArchiveProject::sort_volumes(&mut volumes);

    let order: Vec<&str> = volumes.iter().map(|it| it.to_str().expect("utf-8")).collect();

    assert_eq!(
      order,
      vec![
        "/game/db/addons/mod.db0",
        "/game/db/configs.db0",
        "/game/db/patches/xpatch_1.db",
        // The directory sorts before the file whose name it prefixes, as `xr_strcmp` orders the two entries.
        "/game/db/textures/textures.db0",
        "/game/db/textures.db0",
      ]
    );
  }

  #[test]
  fn recognizes_volume_extensions_without_case() {
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new("game.db0")));
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new("GAME.DB0")));
    assert!(ArchiveDescriptor::is_valid_db_path(Path::new("mod.xdb1")));
    assert!(!ArchiveDescriptor::is_valid_db_path(Path::new("readme.txt")));
    assert!(!ArchiveDescriptor::is_valid_db_path(Path::new("noextension")));
  }

  #[test]
  fn project_root_is_the_common_parent_of_all_archives() {
    let volumes: [PathBuf; 2] = [
      PathBuf::from("/game/database/configs.db0"),
      PathBuf::from("/game/database/patches/patch.db"),
    ];

    assert_eq!(ArchiveProject::root_from_volumes(&volumes), Path::new("/game/database"));
  }
}
