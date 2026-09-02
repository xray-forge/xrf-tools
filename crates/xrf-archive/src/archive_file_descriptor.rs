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
  /// Whether the entry names a directory rather than a file with bytes.
  ///
  /// A volume records the directories it contains so an unpacker can recreate them. X-Ray marks those entries with a
  /// trailing separator; a zero-length entry without one is an empty file.
  pub is_directory: bool,
  /// Entry name as authored, which the engine registers verbatim.
  pub name: String,
  /// Byte offset of the payload inside its volume.
  pub offset: u32,
  /// Payload bytes as stored in the volume.
  pub size_compressed: u32,
  /// Payload bytes once unpacked.
  pub size_real: u32,
  /// Which volume holds the payload, as a position in [`crate::ArchiveProject::archives`].
  ///
  /// A position rather than a path, because an entry belongs to a project and the project already describes each
  /// volume once. Naming the volume again per entry would make every read a search for it, and would let an entry
  /// claim a volume its own project does not hold. The position is also the volume's merge rank, which is what
  /// decides between two entries claiming one name.
  ///
  /// Set by [`crate::ArchiveProject`] as it merges each volume, and stable for the life of that project.
  pub volume: u32,
}

impl ArchiveFileDescriptor {
  /// Creates a descriptor from the fields a name table records.
  ///
  /// The volume is left at zero: a name table says where a payload sits inside its own volume and nothing about which
  /// volume that is, so only the project merging them can answer. [`Self::in_volume`] is where that happens.
  pub fn new(crc: u32, name: String, offset: u32, size_compressed: u32, size_real: u32) -> Self {
    Self {
      crc,
      is_directory: name.ends_with(['\\', '/']),
      name,
      offset,
      size_compressed,
      size_real,
      volume: 0,
    }
  }

  /// The same entry, attributed to the volume at `volume` of its project.
  pub fn in_volume(mut self, volume: u32) -> Self {
    self.volume = volume;

    self
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveFileDescriptor;

  #[test]
  fn an_entry_belongs_to_the_volume_it_is_attributed_to() {
    let descriptor: ArchiveFileDescriptor = ArchiveFileDescriptor::new(0, "configs\\system.ltx".into(), 0, 0, 0);

    // A name table cannot say which volume it is, so a descriptor starts at the first and is placed by its project.
    assert_eq!(descriptor.volume, 0);
    assert_eq!(descriptor.in_volume(3).volume, 3);
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
