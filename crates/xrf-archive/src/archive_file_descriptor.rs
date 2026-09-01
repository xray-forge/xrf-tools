use std::path::Path;
use std::sync::{Arc, LazyLock};

use serde::Serialize;

/// One entry of a volume's name table: where its payload sits and how to verify it.
///
/// Equal `size_real` and `size_compressed` is how the format says "stored uncompressed".
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFileDescriptor {
  /// CRC32 of the unpacked payload, recorded by the packer and verified on decompression.
  pub crc: u32,
  /// The volume file holding the payload.
  ///
  /// Shared with the volume's own descriptor rather than copied: a set has a handful of volumes and tens of thousands
  /// of entries, and every entry of one volume names the same file. Serializes as the path it points at.
  pub source: Arc<Path>,
  /// Root the entry unpacks under, from its volume's header.
  ///
  /// Shared for the same reason as [`Self::source`].
  pub destination: Arc<Path>,
  /// Whether the entry names a directory rather than a file with bytes.
  ///
  /// A volume records the directories it contains so an unpacker can recreate them. X-Ray marks those entries with a
  /// trailing separator; a zero-length entry without one is an empty file.
  pub is_directory: bool,
  /// Entry name as authored, which the engine registers verbatim.
  pub name: String,
  /// Byte offset of the payload inside [`Self::source`].
  pub offset: u32,
  /// Payload bytes as stored in the volume.
  pub size_compressed: u32,
  /// Payload bytes once unpacked.
  pub size_real: u32,
}

/// The placeholder a descriptor carries between the name table and its volume's header.
///
/// Shared rather than allocated per descriptor: the name table is parsed before the header paths are known, so every
/// entry passes through this state on its way to [`ArchiveFileDescriptor::with_archive_paths`].
fn new_empty_path() -> Arc<Path> {
  static EMPTY: LazyLock<Arc<Path>> = LazyLock::new(|| Arc::from(Path::new("")));

  Arc::clone(&EMPTY)
}

impl ArchiveFileDescriptor {
  /// Creates a descriptor from name-table fields, deriving the extension; volume paths attach separately through
  /// [`Self::with_archive_paths`], because the table does not record them.
  pub fn new(crc: u32, name: String, offset: u32, size_compressed: u32, size_real: u32) -> Self {
    Self {
      crc,
      source: new_empty_path(),
      destination: new_empty_path(),
      is_directory: name.ends_with(['\\', '/']),
      name,
      offset,
      size_compressed,
      size_real,
    }
  }

  /// Attaches the volume the entry was read from and the root it unpacks under.
  pub fn with_archive_paths(mut self, source: &Arc<Path>, destination: &Arc<Path>) -> Self {
    self.source = Arc::clone(source);
    self.destination = Arc::clone(destination);
    self
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;
  use std::sync::Arc;

  use super::ArchiveFileDescriptor;

  #[test]
  fn descriptor_shares_the_archive_paths_it_is_given() {
    for name in ["configs\\system.LTX", "scripts/actor.script", "readme"] {
      let source: Arc<Path> = Arc::from(Path::new("database.db0"));
      let destination: Arc<Path> = Arc::from(Path::new("gamedata"));
      let descriptor: ArchiveFileDescriptor =
        ArchiveFileDescriptor::new(0, name.into(), 0, 0, 0).with_archive_paths(&source, &destination);

      assert_eq!(descriptor.source.as_ref(), Path::new("database.db0"));
      assert_eq!(descriptor.destination.as_ref(), Path::new("gamedata"));

      // The point of sharing: an entry carries a refcount on its volume's path, not a copy of it.
      assert!(Arc::ptr_eq(&descriptor.source, &source));
      assert!(Arc::ptr_eq(&descriptor.destination, &destination));
    }
  }

  #[test]
  fn an_entry_with_a_trailing_separator_is_marked_a_directory() {
    for name in ["meshes\\", "meshes\\actors\\", "meshes/actors/"] {
      assert!(
        ArchiveFileDescriptor::new(0, name.into(), 0, 0, 0).is_directory,
        "'{name}' names a directory"
      );
    }

    assert!(!ArchiveFileDescriptor::new(0, "meshes\\empty.ltx".into(), 0, 0, 0).is_directory);
    assert!(!ArchiveFileDescriptor::new(0, "meshes\\actors\\stalker.ogf".into(), 0, 512, 512).is_directory);
  }
}
