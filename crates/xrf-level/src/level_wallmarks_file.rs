use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_format_size};

use crate::level_wallmark_slot::LevelWallmarkSlot;

/// The baked decals of a level, `level.wallmarks`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelWallmarksFile {
  pub slots: Vec<LevelWallmarkSlot>,
}

impl LevelWallmarksFile {
  /// The one chunk the exporter writes.
  pub const SLOTS_CHUNK_ID: u32 = 1;

  /// Reads the decals from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a decal list this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level wallmarks were not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads the decals from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a decal list this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a decal list this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk is absent, or its slots do not account for it.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut slots_reader: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::SLOTS_CHUNK_ID)?;

    let count: u32 = slots_reader.read_u32::<T>()?;
    let mut slots: Vec<LevelWallmarkSlot> =
      slots_reader.new_bounded_vec(count.into(), LevelWallmarkSlot::FIXED_SIZE, "wallmark slots")?;

    for _ in 0..count {
      slots.push(slots_reader.read_xr::<T, _>()?);
    }

    slots_reader.assert_read("Expect all data to be read from level wallmarks chunk")?;

    Ok(Self { slots })
  }

  /// Writes the decals back in the layout the editor exports.
  ///
  /// # Errors
  ///
  /// Returns an error when a count exceeds what the format's `u32` holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut slots: ChunkWriter = ChunkWriter::new();

    slots.write_u32::<T>(to_format_size(self.slots.len(), "wallmark slots")?)?;

    for slot in &self.slots {
      slot.write::<T>(&mut slots)?;
    }

    slots.flush_chunk_into::<T>(&mut writer.buffer, Self::SLOTS_CHUNK_ID)?;

    Ok(())
  }
}

impl LevelWallmarksFile {
  /// Decals across every slot.
  pub fn get_marks_count(&self) -> usize {
    self.slots.iter().map(|slot| slot.marks.len()).sum()
  }

  /// Vertices across every decal, which is what the layer costs to draw.
  pub fn get_vertices_count(&self) -> usize {
    self
      .slots
      .iter()
      .flat_map(|slot| &slot.marks)
      .map(|mark| mark.vertices.len())
      .sum()
  }
}
