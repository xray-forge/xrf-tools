use std::sync::Arc;

use serde::Serialize;

/// One entry of a volume's name table: where its payload sits and how to verify it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFileDescriptor {
  /// Entry name as authored, which the engine registers under its volume's entry point.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub authored_name: Arc<str>,
  /// CRC32 of the unpacked payload, recorded by the packer and verified on decompression.
  pub crc: u32,
  /// Whether the entry names a directory rather than a file with bytes.
  pub is_directory: bool,
  /// Entry name within its project: the authored name behind the part of its volume's entry point the project's other
  /// volumes do not share. A set mounted at one directory, as nearly every set is, names each entry as authored.
  #[cfg_attr(feature = "typescript-bindings", specta(type = String))]
  pub name: Arc<str>,
  /// Byte offset of the payload inside its volume.
  pub offset: u32,
  /// Payload bytes as stored in the volume.
  pub size_compressed: u32,
  /// Payload bytes once unpacked.
  pub size_real: u32,
  /// Which volume holds the payload, as a position in [`crate::ArchiveProject::archives`].
  pub volume: u32,
}

impl ArchiveFileDescriptor {
  /// Creates a descriptor from the fields a name table records.
  pub fn new(crc: u32, name: Arc<str>, offset: u32, size_compressed: u32, size_real: u32) -> Self {
    Self {
      authored_name: Arc::clone(&name),
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

  /// The same entry, named within its project behind `prefix`: the directories, each ending in a separator, its
  /// volume mounts at below the one the project's volumes share.
  pub fn under(mut self, prefix: &str) -> Self {
    if !prefix.is_empty() {
      self.name = format!("{prefix}{}", self.authored_name).into();
    }

    self
  }

  /// Whether the payload is stored LZO-compressed. Equal sizes are how the format says "stored as is".
  pub fn is_compressed(&self) -> bool {
    self.size_compressed != self.size_real
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
  fn an_entry_under_a_prefix_keeps_its_authored_name() {
    let descriptor: ArchiveFileDescriptor =
      ArchiveFileDescriptor::new(0, "common.ltx".into(), 0, 0, 0).under("configs\\");

    assert_eq!(&*descriptor.name, "configs\\common.ltx");
    assert_eq!(&*descriptor.authored_name, "common.ltx");
    assert_eq!(
      &*ArchiveFileDescriptor::new(0, "common.ltx".into(), 0, 0, 0)
        .under("")
        .name,
      "common.ltx"
    );
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

  #[test]
  fn an_entry_is_compressed_when_its_sizes_differ() {
    assert!(ArchiveFileDescriptor::new(0, "configs\\system.ltx".into(), 0, 20, 80).is_compressed());
    assert!(!ArchiveFileDescriptor::new(0, "textures\\wall.dds".into(), 0, 40, 40).is_compressed());
    assert!(!ArchiveFileDescriptor::new(0, "configs\\empty.ltx".into(), 0, 0, 0).is_compressed());
  }
}
