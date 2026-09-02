use std::path::{Path, PathBuf};

use serde::Serialize;

/// One volume of a set: where it is, where it mounts, and what it holds, counted at read time.
///
/// The entries themselves live in [`crate::ArchiveProject::files`] and nowhere else. Retaining a per-volume copy cost
/// one full duplicate of every descriptor in the set, and the only thing that ever read it back was these three
/// totals.
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
  /// The volume file this descriptor was read from.
  pub path: PathBuf,
  /// Bytes this volume's entries occupy as stored, summed while its name table was read.
  pub size_compressed: u64,
  /// Bytes this volume's entries occupy once unpacked, summed while its name table was read.
  pub size_real: u64,
}

impl ArchiveDescriptor {
  /// Whether a path names an archive volume by extension, matching `.db*` and `.xdb*` without case.
  ///
  /// Case-insensitive to agree with the mount planner's volume detection; a non-UTF-8 extension is not a volume rather
  /// than a panic.
  pub fn is_valid_db_path(path: impl AsRef<Path>) -> bool {
    path
      .as_ref()
      .extension()
      .and_then(|extension| extension.to_str())
      .is_some_and(|extension| {
        let extension: String = extension.to_ascii_lowercase();

        extension.starts_with("db") || extension.starts_with("xdb")
      })
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
