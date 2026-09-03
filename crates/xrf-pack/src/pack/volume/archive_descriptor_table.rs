use std::borrow::Cow;
use std::io::Write;

use xrf_archive::{
  CHUNK_HEADER_SIZE, CHUNK_ID_COMPRESSED_MASK, CHUNK_ID_FILE_DESCRIPTORS, DESCRIPTOR_ROW_FIELDS_SIZE,
  DESCRIPTOR_ROW_SIZE_FIELD_SIZE,
};
use xrf_error::{XrfError, XrfResult};
use xrf_lzhuf::compress;
use xrf_utils::{encode_string_to_w1251_bytes, to_format_size};

/// A descriptor row's name, encoded once so the row's cost is known before the row is placed.
///
/// Windows-1251 is one byte per character, so the cost could be counted off the name instead. Encoding answers both
/// questions at once: a name the engine's encoding cannot represent is refused here rather than measured, placed in a
/// volume, and only then rejected — and the size that decided the placement is the size of the bytes actually
/// written, rather than a second calculation that has to be kept agreeing with them.
pub(crate) struct DescriptorName {
  encoded: Vec<u8>,
  /// The row's leading field: the bytes that follow it, which is not the whole row.
  declared_size: u16,
}

impl DescriptorName {
  /// Encode a name as the windows-1251 the engine reads, refusing one no descriptor row can declare.
  pub(crate) fn encode(name: &str) -> XrfResult<Self> {
    let encoded: Vec<u8> = encode_string_to_w1251_bytes(name).map_err(|error| {
      XrfError::new_encoding_error(format!(
        "File name '{name}' cannot be written as windows-1251, which the engine requires: {error}"
      ))
    })?;

    let declared_size: u16 = u16::try_from(encoded.len() + usize::from(DESCRIPTOR_ROW_FIELDS_SIZE))
      .map_err(|_| XrfError::new_invalid_error(format!("File name '{name}' is too long for a descriptor row")))?;

    Ok(Self { encoded, declared_size })
  }

  /// Bytes this name's row costs the table it joins, its leading size field included.
  pub(crate) fn get_row_size(&self) -> u64 {
    u64::from(DESCRIPTOR_ROW_SIZE_FIELD_SIZE) + u64::from(self.declared_size)
  }
}

/// The descriptor table of one volume: the rows `CLocatorAPI` indexes an archive by.
///
/// Owns the row layout, the encoding the engine reads names as, and how the table becomes chunk 1. It also owns the
/// number a volume must reserve for it — [`Self::get_size`] measures the plain table, and [`Self::write_to`] never
/// writes more than that plus a chunk header, which is what lets placement budget a chunk whose coded length it
/// cannot yet know.
pub(crate) struct ArchiveDescriptorTable {
  rows: Vec<u8>,
  /// Length of the directory rows every volume of the set repeats, which [`Self::reset`] rewinds to.
  directories_size: usize,
  /// File rows, as opposed to those directory rows.
  entries: usize,
}

impl ArchiveDescriptorTable {
  /// Cheapest row an entry can cost: the leading size field, the numeric fields, and a one-character name.
  pub(crate) const ROW_SIZE_MIN: u64 = DESCRIPTOR_ROW_SIZE_FIELD_SIZE as u64 + DESCRIPTOR_ROW_FIELDS_SIZE as u64 + 1;

  /// Seed a table with the zero-payload rows that let any single volume list the whole tree.
  pub(crate) fn of_directories(directories: &[String]) -> XrfResult<Self> {
    let mut table: Self = Self {
      rows: Vec::new(),
      directories_size: 0,
      entries: 0,
    };

    for directory in directories {
      // `CLocatorAPI` recognizes archive directories by this delimiter, not by their zero payload.
      table.push_row(&DescriptorName::encode(&format!("{directory}\\"))?, 0, 0, 0, 0);
    }

    table.directories_size = table.rows.len();

    Ok(table)
  }

  /// Drop the file rows, leaving the directory rows the next volume of the set repeats.
  pub(crate) fn reset(&mut self) {
    self.rows.truncate(self.directories_size);
    self.entries = 0;
  }

  /// Record one file: its sizes, checksum, name, and the payload's absolute offset in this volume.
  pub(crate) fn push_entry(
    &mut self,
    name: &DescriptorName,
    size_real: u32,
    size_compressed: u32,
    crc: u32,
    offset: u32,
  ) {
    self.push_row(name, size_real, size_compressed, crc, offset);

    self.entries += 1;
  }

  /// Bytes the plain table occupies, which is what a volume reserves for its descriptor chunk.
  pub(crate) fn get_size(&self) -> u64 {
    self.rows.len() as u64
  }

  /// Files recorded so far, which is what says whether closing this volume could free any room.
  pub(crate) fn get_entries(&self) -> usize {
    self.entries
  }

  /// Write the table as chunk 1 and report the bytes it took.
  pub(crate) fn write_to(&self, writer: &mut impl Write) -> XrfResult<u64> {
    let (chunk_id, payload): (u32, Cow<'_, [u8]>) = self.to_chunk()?;

    writer.write_all(&chunk_id.to_le_bytes())?;
    writer.write_all(&to_format_size::<u32>(payload.len(), "archive descriptor chunk")?.to_le_bytes())?;
    writer.write_all(&payload)?;

    Ok(CHUNK_HEADER_SIZE + payload.len() as u64)
  }

  /// Chunk 1 as it will be written: its id, carrying the compressed mark or not, and its payload.
  ///
  /// Coded the way the engine writes it, unless coding would grow the table. Placement budgeted the plain table, so a
  /// payload longer than that would put the volume past its cap; both readers branch on the mark
  /// (`xray-16/src/xrCore/LocatorAPI.cpp`, `crates/xrf-archive/src/reader.rs`), so the plain form still mounts. It is
  /// the same "compression must pay" rule the payload path applies, and it is what makes [`Self::get_size`] an upper
  /// bound rather than an estimate. An empty table stays plain because LZHUF has no coding for an empty source.
  fn to_chunk(&self) -> XrfResult<(u32, Cow<'_, [u8]>)> {
    if self.rows.is_empty() {
      return Ok((CHUNK_ID_FILE_DESCRIPTORS, Cow::Borrowed(&self.rows)));
    }

    let coded: Vec<u8> = compress(&self.rows)?;

    if coded.len() < self.rows.len() {
      Ok((CHUNK_ID_FILE_DESCRIPTORS | CHUNK_ID_COMPRESSED_MASK, Cow::Owned(coded)))
    } else {
      Ok((CHUNK_ID_FILE_DESCRIPTORS, Cow::Borrowed(&self.rows)))
    }
  }

  /// Append one row, whether it names a directory or a file.
  fn push_row(&mut self, name: &DescriptorName, size_real: u32, size_compressed: u32, crc: u32, offset: u32) {
    self.rows.extend_from_slice(&name.declared_size.to_le_bytes());
    self.rows.extend_from_slice(&size_real.to_le_bytes());
    self.rows.extend_from_slice(&size_compressed.to_le_bytes());
    self.rows.extend_from_slice(&crc.to_le_bytes());
    self.rows.extend_from_slice(&name.encoded);
    self.rows.extend_from_slice(&offset.to_le_bytes());
  }
}

#[cfg(test)]
mod tests {
  use std::borrow::Cow;

  use xrf_archive::{CHUNK_HEADER_SIZE, CHUNK_ID_COMPRESSED_MASK};

  use super::{ArchiveDescriptorTable, DescriptorName};

  fn table_of(directories: &[&str]) -> ArchiveDescriptorTable {
    ArchiveDescriptorTable::of_directories(&directories.iter().map(|name| String::from(*name)).collect::<Vec<_>>())
      .expect("directories encode")
  }

  fn push(table: &mut ArchiveDescriptorTable, name: &str) {
    table.push_entry(&DescriptorName::encode(name).expect("name encodes"), 1, 1, 1, 0);
  }

  fn fill(directories: &[&str], entries: usize) -> ArchiveDescriptorTable {
    let mut table: ArchiveDescriptorTable = table_of(directories);

    for index in 0..entries {
      push(&mut table, &format!("configs\\entry_{index}.ltx"));
    }

    table
  }

  #[test]
  fn a_row_costs_the_table_exactly_what_its_name_promised() {
    let mut table: ArchiveDescriptorTable = table_of(&[]);
    let name: DescriptorName = DescriptorName::encode("configs\\system.ltx").expect("name encodes");

    table.push_entry(&name, 1, 1, 1, 0);

    // The number placement reserves and the number the table grows by must be one number. They were two before
    // `issues/closed/0039`: the row's leading field declares what follows it, so a row is two bytes wider than the
    // size it states, and a volume closed that much past its cap for every entry it held.
    assert_eq!(table.get_size(), name.get_row_size());
    assert_eq!(name.get_row_size(), "configs\\system.ltx".len() as u64 + 18);
  }

  #[test]
  fn a_row_declares_the_size_the_engine_reads_it_by() {
    let name: DescriptorName = DescriptorName::encode("a").expect("name encodes");
    let mut table: ArchiveDescriptorTable = table_of(&[]);

    table.push_entry(&name, 1, 1, 1, 0);

    // `archive_file_header` reads this field as the bytes following it, so it must exclude its own two.
    let declared: u16 = u16::from_le_bytes(table.rows[..2].try_into().expect("size field"));

    assert_eq!(u64::from(declared), table.get_size() - 2);
    assert_eq!(
      ArchiveDescriptorTable::ROW_SIZE_MIN,
      name.get_row_size(),
      "and a one-character name is the cheapest row"
    );
  }

  #[test]
  fn a_cyrillic_name_costs_one_byte_per_character() {
    // Windows-1251 is single byte, so the row is shorter than the UTF-8 the name arrived as.
    let name: DescriptorName = DescriptorName::encode("configs\\текст.ltx").expect("name encodes");

    assert_eq!(name.get_row_size(), "configs\\текст.ltx".chars().count() as u64 + 18);
  }

  #[test]
  fn a_name_the_engine_could_not_read_never_reaches_a_row() {
    assert!(DescriptorName::encode("configs\\ロゴ.ltx").is_err());
  }

  #[test]
  fn resetting_keeps_the_directory_rows_every_volume_repeats() {
    let mut table: ArchiveDescriptorTable = fill(&["configs", "textures"], 0);
    let directories: u64 = table.get_size();

    push(&mut table, "configs\\system.ltx");

    assert_eq!(table.get_entries(), 1);
    assert!(table.get_size() > directories);

    table.reset();

    assert_eq!(table.get_entries(), 0);
    assert_eq!(table.get_size(), directories, "the next volume still lists the tree");
  }

  #[test]
  fn a_written_chunk_never_outgrows_the_size_placement_reserved() {
    // The whole cap guarantee rests on this: placement budgets `size()` for a chunk whose coded length it cannot yet
    // know, so no table may write more than that plus its chunk header.
    for entries in [0, 1, 2, 8, 64, 512] {
      let table: ArchiveDescriptorTable = fill(&["configs"], entries);
      let mut written: Vec<u8> = Vec::new();
      let reported: u64 = table.write_to(&mut written).expect("table writes");

      assert_eq!(reported, written.len() as u64, "the report is the bytes written");
      assert!(
        reported <= CHUNK_HEADER_SIZE + table.get_size(),
        "a table of {} row byte(s) wrote {reported}",
        table.get_size()
      );
    }
  }

  #[test]
  fn the_written_form_is_never_the_larger_of_the_two() {
    // Whichever way the table happens to code at a given size, the mark must describe what was written and the
    // payload must not exceed the plain table. Sizes are swept rather than picked, because where coding starts to
    // pay is a property of the coder and not something this rule may depend on.
    for entries in 0..24 {
      let table: ArchiveDescriptorTable = fill(&["configs"], entries);
      let (chunk_id, payload): (u32, Cow<'_, [u8]>) = table.to_chunk().expect("table codes");

      assert_eq!(
        chunk_id & CHUNK_ID_COMPRESSED_MASK != 0,
        (payload.len() as u64) < table.get_size(),
        "the mark says what {entries} entrie(s) were written as"
      );
      assert!(payload.len() as u64 <= table.get_size());
    }
  }

  #[test]
  fn an_empty_table_is_written_plain_rather_than_coded() {
    // LZHUF refuses an empty source, so asking it would fail rather than fall back. Chunk 1 with a zero payload is
    // what a reader walks past.
    let mut written: Vec<u8> = Vec::new();

    table_of(&[]).write_to(&mut written).expect("table writes");

    assert_eq!(written, [1, 0, 0, 0, 0, 0, 0, 0]);
  }
}
