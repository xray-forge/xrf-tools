use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;

use crate::archive_file_descriptor::ArchiveFileDescriptor;

/// The fields a reader locates and checks a payload by: volume, offset, stored size, real size, and CRC.
type PayloadLocation = (u32, u32, u32, u32, u32);

/// Stored bytes that several file entries of one volume set locate at once.
///
/// Derived from the descriptors, never recorded by a writer: the format has no alias field, so a packer that stored a
/// file once and pointed a second row at it left only equal fields behind. Calling this "aliased" would claim to know
/// what the packer did; it knows only what a reader does, which is read the same bytes for every name here.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSharedPayload {
  /// Which volume holds the bytes, as a position in [`crate::ArchiveProject::archives`].
  pub volume: u32,
  /// Byte offset of the payload inside its volume.
  pub offset: u32,
  /// Payload bytes as stored in the volume.
  pub size_compressed: u32,
  /// Payload bytes once unpacked.
  pub size_real: u32,
  /// CRC32 of the unpacked payload.
  pub crc: u32,
  /// Authored names of every file entry located here, in name order; always two or more.
  #[cfg_attr(feature = "typescript-bindings", specta(type = Vec<String>))]
  pub names: Vec<Arc<str>>,
}

impl ArchiveSharedPayload {
  /// The payloads more than one file entry locates, in volume and offset order.
  ///
  /// Grouped by every field a reader locates and checks a payload with, so two entries in one group read identical
  /// bytes whatever wrote them. Directory rows are left out: they carry no payload, and every one in a volume would
  /// otherwise "share" the same nothing.
  pub fn derive<'a>(entries: impl IntoIterator<Item = &'a ArchiveFileDescriptor>) -> Vec<Self> {
    let mut groups: HashMap<PayloadLocation, Vec<Arc<str>>> = HashMap::new();

    for entry in entries {
      if entry.is_directory {
        continue;
      }

      groups
        .entry((
          entry.volume,
          entry.offset,
          entry.size_compressed,
          entry.size_real,
          entry.crc,
        ))
        .or_default()
        .push(entry.name.clone());
    }

    let mut payloads: Vec<Self> = groups
      .into_iter()
      .filter(|(_, names)| names.len() > 1)
      .map(|((volume, offset, size_compressed, size_real, crc), mut names)| {
        names.sort_unstable();

        Self {
          volume,
          offset,
          size_compressed,
          size_real,
          crc,
          names,
        }
      })
      .collect();

    payloads.sort_unstable_by_key(|payload| (payload.volume, payload.offset));
    payloads
  }

  /// Whether `entry` is one of the rows located here.
  pub fn locates(&self, entry: &ArchiveFileDescriptor) -> bool {
    !entry.is_directory
      && self.volume == entry.volume
      && self.offset == entry.offset
      && self.size_compressed == entry.size_compressed
      && self.size_real == entry.size_real
      && self.crc == entry.crc
  }

  /// The names located here other than `name`, in name order.
  pub fn get_others_of(&self, name: &str) -> impl Iterator<Item = &Arc<str>> {
    self.names.iter().filter(move |it| it.as_ref() != name)
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::archive_file_descriptor::ArchiveFileDescriptor;

  use super::ArchiveSharedPayload;

  fn entry(
    name: &str,
    volume: u32,
    offset: u32,
    size_compressed: u32,
    size_real: u32,
    crc: u32,
  ) -> ArchiveFileDescriptor {
    ArchiveFileDescriptor::new(crc, Arc::from(name), offset, size_compressed, size_real).in_volume(volume)
  }

  #[test]
  fn groups_entries_a_reader_would_locate_at_the_same_bytes() {
    let entries: [ArchiveFileDescriptor; 4] = [
      entry("configs\\third.ltx", 0, 64, 20, 80, 7),
      entry("configs\\first.ltx", 0, 64, 20, 80, 7),
      entry("configs\\second.ltx", 0, 64, 20, 80, 7),
      entry("textures\\wall.dds", 0, 84, 40, 40, 9),
    ];

    let payloads: Vec<ArchiveSharedPayload> = ArchiveSharedPayload::derive(&entries);

    assert_eq!(payloads.len(), 1, "the texture has its own bytes");
    assert_eq!(
      payloads[0].names,
      vec![
        Arc::from("configs\\first.ltx"),
        Arc::from("configs\\second.ltx"),
        Arc::from("configs\\third.ltx")
      ],
      "names are sorted rather than left in map order"
    );
    assert_eq!(
      (payloads[0].offset, payloads[0].size_compressed, payloads[0].crc),
      (64, 20, 7)
    );
    assert!(payloads[0].locates(&entries[1]));
    assert!(!payloads[0].locates(&entries[3]));
    assert_eq!(
      payloads[0].get_others_of("configs\\second.ltx").collect::<Vec<_>>(),
      vec![&Arc::from("configs\\first.ltx"), &Arc::from("configs\\third.ltx")]
    );
  }

  #[test]
  fn an_equal_location_in_another_volume_is_another_payload() {
    // An offset addresses one volume, so the same numbers in two volumes are two payloads that happen to agree.
    let entries: [ArchiveFileDescriptor; 4] = [
      entry("a.dds", 0, 64, 40, 40, 9),
      entry("b.dds", 0, 64, 40, 40, 9),
      entry("c.dds", 1, 64, 40, 40, 9),
      entry("d.dds", 1, 64, 40, 40, 9),
    ];

    let payloads: Vec<ArchiveSharedPayload> = ArchiveSharedPayload::derive(&entries);

    assert_eq!(payloads.len(), 2);
    assert_eq!(
      (payloads[0].volume, payloads[1].volume),
      (0, 1),
      "ordered by volume, then offset"
    );
    assert_eq!(payloads[0].names, vec![Arc::from("a.dds"), Arc::from("b.dds")]);
    assert_eq!(payloads[1].names, vec![Arc::from("c.dds"), Arc::from("d.dds")]);
  }

  #[test]
  fn a_field_apart_is_not_shared() {
    // Every field a reader checks has to agree: an empty file and the entry written right after it share an offset
    // and nothing else, and two payloads of one size and checksum at different offsets are two payloads.
    let entries: [ArchiveFileDescriptor; 4] = [
      entry("configs\\empty.ltx", 0, 64, 0, 0, 0),
      entry("configs\\system.ltx", 0, 64, 20, 80, 7),
      entry("textures\\a.dds", 0, 84, 40, 40, 9),
      entry("textures\\b.dds", 0, 124, 40, 40, 9),
    ];

    assert!(ArchiveSharedPayload::derive(&entries).is_empty());
  }

  #[test]
  fn directory_rows_share_nothing() {
    let entries: [ArchiveFileDescriptor; 3] = [
      entry("configs\\", 0, 0, 0, 0, 0),
      entry("textures\\", 0, 0, 0, 0, 0),
      entry("configs\\empty.ltx", 0, 0, 0, 0, 0),
    ];

    assert!(
      ArchiveSharedPayload::derive(&entries).is_empty(),
      "a payload-less row is not a payload two names share"
    );
  }

  #[test]
  fn empty_files_at_one_offset_are_one_payload() {
    // xrCompress and xrf-pack both alias a second empty file onto the first, so the reader sees two zero-length rows
    // at one offset, which is a shared payload of nothing.
    let entries: [ArchiveFileDescriptor; 2] = [
      entry("configs\\a_empty.ltx", 0, 64, 0, 0, 0),
      entry("configs\\b_empty.ltx", 0, 64, 0, 0, 0),
    ];

    assert_eq!(ArchiveSharedPayload::derive(&entries).len(), 1);
  }
}
