use std::fmt::Debug;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveDescriptor;
use xrf_error::{XrfError, XrfResult};

use crate::source::{XrayDeclaredRoot, XrayFileStamp, XraySourceKind};
use crate::{XrayAssetContainer, XrayPathCollision, XrayShadowedCopy};

/// Names a source after the directory or volume set it was opened from, for [`XrayAssetSource::get_label`].
pub fn label_from_path(path: &Path) -> String {
  path
    .file_name()
    .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().to_string())
}

/// An asset source addressed by normalized paths relative to itself.
pub trait XrayAssetSource: Debug + Send + Sync {
  /// Short name for reporting, such as a directory or volume-set name.
  fn get_label(&self) -> &str;

  /// Classifies the source's physical storage.
  fn get_kind(&self) -> XraySourceKind;

  /// Whether a commit may write, create and delete files in this source, each at [`Self::get_writable_path`].
  fn is_writable(&self) -> bool;

  /// Returns the directory root or the directory containing the archive volumes.
  fn get_root_path(&self) -> &Path;

  /// Checks whether the source contains a source-relative logical path.
  fn contains(&self, path: &str) -> bool {
    self.locate(path).is_some()
  }

  /// Locates an entry in its physical container.
  fn locate(&self, path: &str) -> Option<XrayAssetContainer>;

  /// Reads an existing entry.
  fn read(&self, path: &str) -> XrfResult<Vec<u8>>;

  /// Where a writable source keeps a source-relative path on disk: the file it holds for it, or where a new one goes.
  /// `None` for a source nothing writes to.
  fn get_writable_path(&self, path: &str) -> Option<PathBuf> {
    let _ = path;

    None
  }

  /// This source once the `created` paths were added on disk and the `deleted` ones removed, without reading anything
  /// it holds again.
  ///
  /// # Errors
  ///
  /// Returns an error when a path is not a valid X-Ray logical path, or when the source cannot take changes in place,
  /// as a set of archive volumes cannot.
  fn with_changes(&self, created: &[&str], deleted: &[&str]) -> XrfResult<Box<dyn XrayAssetSource>> {
    let _ = (created, deleted);

    Err(XrfError::new_not_implemented_error(format!(
      "source '{}' cannot take changes in place",
      self.get_label()
    )))
  }

  /// Enumerates source-relative logical paths, optionally restricted to a component prefix.
  fn list_entries<'a>(&'a self, prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a>;

  /// How many engine identities this source answers for, before anything mounted in front of it shadows one.
  fn count_entries(&self) -> usize {
    self.list_entries(None).count()
  }

  /// The volumes this source merged into one name table, in merge order — a later one wins.
  fn list_volumes(&self) -> &[ArchiveDescriptor] {
    &[]
  }

  /// Size in bytes of an entry this source holds, without reading it.
  fn get_size(&self, path: &str) -> Option<u64>;

  /// How an entry's file looks on disk now, for a source whose files can change behind it; `None` where entries never
  /// change in place, or where the file cannot be looked at.
  fn get_stamp(&self, path: &str) -> Option<XrayFileStamp> {
    let _ = path;

    None
  }

  /// CRC32 of an entry's unpacked payload, when the source already knows it without reading anything.
  fn get_recorded_crc(&self, path: &str) -> Option<u32> {
    let _ = path;

    None
  }

  /// Where this source's own metadata says its entries mount, for each container that declares it.
  fn list_declared_roots(&self) -> Vec<XrayDeclaredRoot> {
    Vec::new()
  }

  /// Files this source holds but cannot reach, because another file already claims their engine identity.
  fn get_collisions(&self) -> &[XrayPathCollision] {
    &[]
  }

  /// Copies this source holds behind the one it answers with, because its own ordering put another in front.
  fn list_shadowed(&self) -> &[XrayShadowedCopy] {
    &[]
  }
}
