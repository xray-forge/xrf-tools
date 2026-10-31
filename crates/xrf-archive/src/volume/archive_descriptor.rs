use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_extension::get_path_extension;

/// One volume of a set: where it is, where it mounts, and what it holds, counted at read time.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDescriptor {
  /// Volume file creation time in Unix milliseconds, when the filesystem reports one.
  pub created_at: Option<u64>,
  /// Volume file modification time in Unix milliseconds, when the filesystem reports one.
  pub modified_at: Option<u64>,
  /// Entries this volume's name table holds, before any merge shadows one of them.
  pub entries: usize,
  /// Root the volume unpacks under, from `[header] entry_point` with its alias stripped.
  pub output_root_path: PathBuf,
  /// Directory under `$fs_root$` the engine registers this volume's entries in, with backslash separators and none at
  /// either end: the entry point without its alias, which always stands for `$fs_root$` there, or `gamedata` for a
  /// volume without metadata (`CLocatorAPI::LoadArchive`). Unlike `output_root_path`, it tells those two apart from an
  /// entry point of `$fs_root$\` itself.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub mount_root: String,
  /// The volume file this descriptor was read from.
  pub path: PathBuf,
  /// Bytes this volume's entries occupy as stored, summed while its name table was read.
  pub size_compressed: u64,
  /// Bytes this volume's entries occupy once unpacked, summed while its name table was read.
  pub size_real: u64,
}

impl ArchiveDescriptor {
  /// Whether a path names an archive volume by its extension alone, matching `.db*` and `.xdb*` without case.
  pub fn is_volume_path(path: impl AsRef<Path>) -> bool {
    get_path_extension(path.as_ref()).is_some_and(|extension| {
      let extension: String = extension.to_ascii_lowercase();

      extension.starts_with("db") || extension.starts_with("xdb")
    })
  }

  /// The directories of [`Self::mount_root`], outermost first.
  pub fn list_mount_directories(&self) -> impl Iterator<Item = &str> {
    self.mount_root.split('\\').filter(|directory| !directory.is_empty())
  }

  /// The directories of [`Self::mount_root`] after its first `outer`, each ending in a separator: the prefix that puts
  /// an authored name where this volume mounts it, seen from that outer directory.
  pub fn to_mount_prefix_below(&self, outer: usize) -> String {
    self
      .list_mount_directories()
      .skip(outer)
      .map(|directory| format!("{directory}\\"))
      .collect()
  }

  /// Bytes this volume's entries occupy once unpacked.
  pub fn get_real_size(&self) -> u64 {
    self.size_real
  }

  /// Bytes this volume's entries occupy as stored.
  pub fn get_compressed_size(&self) -> u64 {
    self.size_compressed
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveDescriptor;

  #[test]
  fn recognizes_volume_extensions_without_case() {
    assert!(ArchiveDescriptor::is_volume_path("game.db0"));
    assert!(ArchiveDescriptor::is_volume_path("GAME.DB0"));
    assert!(ArchiveDescriptor::is_volume_path("mod.xdb1"));
    assert!(!ArchiveDescriptor::is_volume_path("readme.txt"));
    assert!(!ArchiveDescriptor::is_volume_path("noextension"));
  }

  #[test]
  fn recognizes_a_volume_whose_whole_name_is_its_extension() {
    assert!(ArchiveDescriptor::is_volume_path("db/.db0"));
    assert!(ArchiveDescriptor::is_volume_path(".xdb1"));

    // A dot in a directory name is still not the extension of an extensionless file.
    assert!(!ArchiveDescriptor::is_volume_path("game.db0/header"));
  }
}
