use xrf_archive::{CHUNK_HEADER_SIZE, CHUNK_SIZE_FIELD_SIZE};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::encode_string_to_w1251_bytes;

use crate::pack::archive_descriptor_table::ArchiveDescriptorTable;
use crate::pack::archive_pack_config::ArchivePackConfig;

/// What every volume of one archive set costs before it holds anything, and the size it must close within.
pub(crate) struct ArchiveVolumeLayout {
  maximum: u64,
  /// `[header]` text as chunk 666 carries it, encoded once because every volume of the set repeats it.
  header: Option<Vec<u8>>,
  opening: u64,
}

impl ArchiveVolumeLayout {
  /// Measure what every volume repeats, refusing a cap those parts already fill.
  pub(crate) fn new(config: &ArchivePackConfig, descriptors: &ArchiveDescriptorTable) -> XrfResult<Self> {
    let header: Option<Vec<u8>> = config
      .header
      .as_deref()
      .map(|header| {
        encode_string_to_w1251_bytes(header).map_err(|error| {
          XrfError::new_encoding_error(format!("Failed to encode archive header as windows-1251: {error}"))
        })
      })
      .transpose()?;

    let layout: Self = Self {
      maximum: config.max_volume_size,
      opening: header
        .as_ref()
        .map_or(0, |header| CHUNK_HEADER_SIZE + header.len() as u64)
        + CHUNK_HEADER_SIZE,
      header,
    };

    // Splitting is what answers a full volume, and it cannot answer this: every volume of the set opens with the same
    // repeated parts, so if they leave no room for one entry, the next volume has none either. Refused before a
    // destination exists, so the failure names the configuration rather than whichever file happened to arrive first.
    let smallest: u64 = Self::get_closed_size(
      layout.opening,
      descriptors.get_size() + ArchiveDescriptorTable::ROW_SIZE_MIN,
    );

    if smallest > layout.maximum {
      return Err(XrfError::new_invalid_error(format!(
        "A volume of this archive needs {smallest} bytes for its chunk headers, header text and directory rows before \
         it can hold even one entry, past the configured maximum volume size of {} bytes",
        layout.maximum
      )));
    }

    Ok(layout)
  }

  /// Bytes a volume occupies once closed, from how far its payloads reach and how large its table is.
  ///
  /// The descriptor chunk is measured by the plain table, which [`ArchiveDescriptorTable::write_to`] never exceeds, so
  /// this is an upper bound on a coded length that placement cannot yet know rather than an estimate of it.
  pub(crate) fn get_closed_size(position: u64, table_size: u64) -> u64 {
    position + CHUNK_HEADER_SIZE + table_size
  }

  /// Bytes a volume holds the moment it opens: the optional header chunk, then the data chunk's id and blank size.
  pub(crate) fn get_opening_size(&self) -> u64 {
    self.opening
  }

  /// Where the data chunk's size field sits, being the last field of the opening, patched once its payloads are in.
  pub(crate) fn get_data_size_position(&self) -> u64 {
    self.opening - CHUNK_SIZE_FIELD_SIZE
  }

  pub(crate) fn get_header(&self) -> Option<&[u8]> {
    self.header.as_deref()
  }

  pub(crate) fn get_maximum(&self) -> u64 {
    self.maximum
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;

  use super::{ArchiveVolumeLayout, CHUNK_HEADER_SIZE};
  use crate::pack::archive_descriptor_table::ArchiveDescriptorTable;
  use crate::pack::archive_pack_config::ArchivePackConfig;

  fn layout_of(max_volume_size: u64, header: Option<&str>) -> Result<ArchiveVolumeLayout, XrfError> {
    let mut config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "packed");

    config.max_volume_size = max_volume_size;
    config.header = header.map(String::from);

    ArchiveVolumeLayout::new(
      &config,
      &ArchiveDescriptorTable::of_directories(&[]).expect("empty table"),
    )
  }

  #[test]
  fn an_opening_is_the_header_chunk_and_the_data_chunk_header() {
    let bare: ArchiveVolumeLayout = layout_of(1024, None).expect("cap holds an entry");
    let headed: ArchiveVolumeLayout = layout_of(1024, Some("[header]\r\n")).expect("cap holds an entry");

    assert_eq!(bare.get_opening_size(), CHUNK_HEADER_SIZE);
    assert_eq!(
      headed.get_opening_size(),
      CHUNK_HEADER_SIZE * 2 + "[header]\r\n".len() as u64
    );

    // The size field is the last of those bytes, which is where the payload total is patched back in.
    assert_eq!(headed.get_data_size_position(), headed.get_opening_size() - 4);
  }

  #[test]
  fn a_closed_volume_counts_its_descriptor_chunk() {
    // The overshoot 0039 recorded was exactly this chunk going unaccounted for.
    assert_eq!(ArchiveVolumeLayout::get_closed_size(100, 30), 138);
  }

  #[test]
  fn a_cap_with_no_room_for_one_entry_is_refused_before_anything_is_written() {
    let table: ArchiveDescriptorTable = ArchiveDescriptorTable::of_directories(&[]).expect("empty table");
    let smallest: u64 = ArchiveVolumeLayout::get_closed_size(
      CHUNK_HEADER_SIZE,
      table.get_size() + ArchiveDescriptorTable::ROW_SIZE_MIN,
    );

    assert!(layout_of(smallest, None).is_ok(), "a cap of exactly {smallest} fits");
    assert!(matches!(layout_of(smallest - 1, None), Err(XrfError::Invalid { .. })));
  }

  #[test]
  fn directory_rows_count_against_the_cap_in_every_volume() {
    let mut config: ArchivePackConfig = ArchivePackConfig::new("gamedata", "db", "packed");
    let table: ArchiveDescriptorTable =
      ArchiveDescriptorTable::of_directories(&[String::from("configs")]).expect("directories encode");

    config.header = None;
    config.max_volume_size = ArchiveVolumeLayout::get_closed_size(
      CHUNK_HEADER_SIZE,
      table.get_size() + ArchiveDescriptorTable::ROW_SIZE_MIN,
    ) - 1;

    // The same cap holds an archive that lists no directories and not one that lists `configs`.
    assert!(ArchiveVolumeLayout::new(&config, &table).is_err());
    assert!(
      ArchiveVolumeLayout::new(
        &config,
        &ArchiveDescriptorTable::of_directories(&[]).expect("empty table")
      )
      .is_ok()
    );
  }
}
