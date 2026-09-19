use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_format_size};

use crate::archive_read_policy::ArchiveReadPolicy;
use crate::archive_read_result::ArchiveReadResult;
use crate::archive_volume_discovery::ArchiveVolumeDiscovery;
use crate::payload::ArchiveOpenVolumes;
use crate::payload::ArchiveSharedPayload;
use crate::volume::ArchiveDescriptor;
use crate::volume::ArchiveFileDescriptor;
use crate::volume::ArchiveReader;

/// One volume set at a path the caller names, merged into a single name table.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveProject {
  /// Volumes in merge order: a later one wins the name table, so a caller searching them as separate sources must
  /// search them in reverse to resolve an entry to the bytes this project's table names.
  pub archives: Vec<ArchiveDescriptor>,
  /// Entries keyed by their authored name, which is the same allocation each descriptor carries as its `name`.
  #[cfg_attr(feature = "typescript-bindings", specta(type = HashMap<String, ArchiveFileDescriptor>))]
  pub files: HashMap<Arc<str>, ArchiveFileDescriptor>,
  /// Entries a later volume overwrote in the merge, in the order they were displaced.
  pub shadowed: Vec<ArchiveFileDescriptor>,
  pub read_policy: ArchiveReadPolicy,
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
  /// Returns an error when the path cannot be walked, no archive volume is found, or a volume cannot be read.
  pub fn new(path: impl AsRef<Path>) -> XrfResult<Self> {
    Self::read_to_depth(path.as_ref(), usize::MAX)
  }

  /// Reads one archive file or archive volumes directly under a directory.
  ///
  /// # Errors
  ///
  /// Returns an error when the path cannot be walked, no archive volume is found, or a volume cannot be read.
  pub fn new_shallow(path: impl AsRef<Path>) -> XrfResult<Self> {
    Self::read_to_depth(path.as_ref(), 1)
  }

  fn read_to_depth(path: &Path, depth: usize) -> XrfResult<Self> {
    let mut files: HashMap<Arc<str>, ArchiveFileDescriptor> = HashMap::new();
    let mut shadowed: Vec<ArchiveFileDescriptor> = Vec::new();
    let is_single_volume: bool = path.is_file();

    if !is_single_volume {
      log::info!("Reading archive directory: {}", format_path(path));
    }

    let volumes: Vec<PathBuf> = ArchiveVolumeDiscovery::discover(path, depth)?;
    let mut archives: Vec<ArchiveDescriptor> = Vec::with_capacity(volumes.len());

    for volume in &volumes {
      log::info!("Reading archive file: {}", format_path(volume));

      let (descriptor, entries) = ArchiveReader::from_path(volume)?.read_archive()?;
      // A set of more than `u32::MAX` volumes cannot be described by the format, and a silent narrowing here would
      // attribute entries to the wrong file rather than refusing the set.
      let index: u32 = to_format_size(archives.len(), "archive volume count")?;

      // Volumes are read in merge order, so a later one overwrites the name an earlier one claimed. The displaced
      // entry is kept rather than dropped: it is the copy a patch volume replaced, and nothing downstream can
      // reconstruct it once the table has folded. Attributing each entry here is the only point that knows both the
      // entry and which volume of the set it arrived from.
      for (name, entry) in entries {
        // A displaced directory row is not a hidden copy of anything: every volume records the directories it
        // contains, so keeping those would retain one per shared directory per volume and say nothing.
        if let Some(displaced) = files.insert(name, entry.in_volume(index))
          && !displaced.is_directory
        {
          shadowed.push(displaced);
        }
      }

      archives.push(descriptor);
    }

    if archives.is_empty() {
      return Err(XrfError::new_read_error(format!(
        "Unable to read archives at location {}",
        format_path(path)
      )));
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
      read_policy: ArchiveReadPolicy::default(),
      root,
      shadowed,
      size_real,
    })
  }

  /// The volumes [`Self::new`] would read at a path, in the order it merges them.
  ///
  /// # Errors
  ///
  /// Returns an error when a descendant of the path cannot be walked.
  pub fn get_discover_volumes(path: impl AsRef<Path>) -> XrfResult<Vec<PathBuf>> {
    ArchiveVolumeDiscovery::discover(path.as_ref(), usize::MAX)
  }

  /// The volume an entry's payload sits in.
  ///
  /// # Errors
  ///
  /// Returns a read error when the entry names a volume outside this project.
  pub fn get_volume_of(&self, descriptor: &ArchiveFileDescriptor) -> XrfResult<&ArchiveDescriptor> {
    self.archives.get(descriptor.volume as usize).ok_or_else(|| {
      XrfError::new_read_error(format!(
        "entry '{}' names volume {}, and its project holds {}",
        descriptor.name,
        descriptor.volume,
        self.archives.len()
      ))
    })
  }

  /// Bytes the merged name table's entries occupy once unpacked.
  pub fn get_real_size(&self) -> u64 {
    self.size_real
  }

  /// Bytes the merged name table's entries occupy as stored.
  pub fn get_compressed_size(&self) -> u64 {
    self.files.values().map(|file| u64::from(file.size_compressed)).sum()
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

  /// Opens every volume of the set, once, for as long as the returned value lives.
  ///
  /// # Errors
  ///
  /// Returns an IO error when a volume cannot be opened or its length cannot be read.
  pub fn open_volumes(&self) -> XrfResult<ArchiveOpenVolumes<'_>> {
    ArchiveOpenVolumes::open(self)
  }

  /// Read one archived file into memory, decompressing it when it is stored compressed.
  pub fn read_file_bytes(&self, name: &str) -> XrfResult<Vec<u8>> {
    let descriptor: &ArchiveFileDescriptor = self
      .files
      .get(name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Cannot read '{name}' - no such file in the archive.")))?;

    self.open_volumes()?.read_bytes(descriptor)
  }

  /// Read one archived file as text, subject to the project's read policy.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry is absent, the policy refuses it, or its bytes cannot be read or decoded.
  pub fn read_file_as_string(&self, filename: &str) -> XrfResult<ArchiveReadResult> {
    log::info!("Trying to read file from archive: {filename}");

    let descriptor: &ArchiveFileDescriptor = self
      .files
      .get(filename)
      .ok_or_else(|| XrfError::new_not_found_error(format!("File '{filename}' is not found in the archive project")))?;

    self
      .read_policy
      .assert_can_read_as_text(filename, descriptor.size_real)?;

    ArchiveReadResult::decode(filename, &self.read_file_bytes(filename)?, descriptor.size_real)
  }

  /// The payloads more than one entry of this project locates, in volume and offset order.
  pub fn list_shared_payloads(&self) -> Vec<ArchiveSharedPayload> {
    ArchiveSharedPayload::derive(self.files.values())
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use super::ArchiveProject;

  #[test]
  fn project_root_is_the_common_parent_of_all_archives() {
    let volumes: [PathBuf; 2] = [
      PathBuf::from("/game/database/configs.db0"),
      PathBuf::from("/game/database/patches/patch.db"),
    ];

    assert_eq!(ArchiveProject::root_from_volumes(&volumes), Path::new("/game/database"));
  }
}
