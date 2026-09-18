use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::details::detail_model::DetailModel;
use crate::details::level_details_slot::LevelDetailsSlot;

/// Metres one slot of the detail grid covers, `DETAIL_SLOT_SIZE` (`Layers/xrRender/DetailFormat.h`).
pub const DETAIL_SLOT_METERS: f32 = 2.0;

/// The 24 bytes `CDetailManager::Load` casts onto, `DetailHeader`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDetailsHeader {
  pub version: u32,
  /// Detail objects the library holds, which the slot grid indexes into with six bits.
  pub object_count: u32,
  pub offset_x: i32,
  pub offset_z: i32,
  pub size_x: u32,
  pub size_z: u32,
}

impl LevelDetailsHeader {
  /// Byte size of the header as laid out by the engine.
  pub const SIZE: u64 = 24;
}

impl ChunkReadWrite for LevelDetailsHeader {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u32::<T>()?,
      object_count: reader.read_u32::<T>()?,
      offset_x: reader.read_i32::<T>()?,
      offset_z: reader.read_i32::<T>()?,
      size_x: reader.read_u32::<T>()?,
      size_z: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_u32::<T>(self.object_count)?;
    writer.write_i32::<T>(self.offset_x)?;
    writer.write_i32::<T>(self.offset_z)?;
    writer.write_u32::<T>(self.size_x)?;
    writer.write_u32::<T>(self.size_z)?;

    Ok(())
  }
}

/// The detail object library and planting grid of a level, `level.details`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDetailsFile {
  pub header: LevelDetailsHeader,
  /// The library the grid indexes into, in the order the file numbers its chunks.
  pub objects: Vec<DetailModel>,
  /// The grid as stored, `size_x * size_z` records of [`LevelDetailsSlot::SERIALIZED_SIZE`] bytes.
  #[serde(skip)]
  pub slots: Vec<u8>,
}

impl LevelDetailsFile {
  /// The version `CDetailManager::Load` asserts on, `DETAIL_VERSION`.
  pub const CURRENT_VERSION: u32 = 3;

  /// `DETMGR_CHUNK_HEADER`.
  pub const HEADER_CHUNK_ID: u32 = 0;

  /// `DETMGR_CHUNK_OBJECTS`, whose children are numbered by library index.
  pub const OBJECTS_CHUNK_ID: u32 = 1;

  /// `DETMGR_CHUNK_SLOTS`, whose payload is the grid and nothing else.
  pub const SLOTS_CHUNK_ID: u32 = 2;

  /// The largest library index the grid's six bits can address, one of which means "nothing".
  pub const MAXIMUM_OBJECTS: u32 = 63;

  /// Reads a detail library from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a detail library this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level details file was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a detail library from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a detail library this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived library arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a detail library this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is absent, the version is one this does not read, the object chunk does not hold
  /// exactly what the header declares, or the grid is not the size the header gives it.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let header: LevelDetailsHeader =
      find_required_chunk_by_id(&chunks, Self::HEADER_CHUNK_ID)?.read_xr::<T, LevelDetailsHeader>()?;

    if header.version != Self::CURRENT_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected level details version {} on read, only version {} is implemented",
        header.version,
        Self::CURRENT_VERSION
      )));
    }

    let mut objects: Vec<DetailModel> = Vec::new();

    for mut chunk in find_required_chunk_by_id(&chunks, Self::OBJECTS_CHUNK_ID)?.read_children()? {
      objects.push(DetailModel::read::<T, D>(&mut chunk)?);
      chunk.assert_read("Expect all data to be read from detail object chunk")?;
    }

    if objects.len() as u64 != u64::from(header.object_count) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level details object count {}, header declares {}",
        objects.len(),
        header.object_count
      )));
    }

    let slots: Vec<u8> = find_required_chunk_by_id(&chunks, Self::SLOTS_CHUNK_ID)?.read_remaining()?;
    let expected: u64 = u64::from(header.size_x) * u64::from(header.size_z) * LevelDetailsSlot::SERIALIZED_SIZE as u64;

    if slots.len() as u64 != expected {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level details grid of {} bytes, a {}x{} grid is {expected}",
        slots.len(),
        header.size_x,
        header.size_z
      )));
    }

    Ok(Self { header, objects, slots })
  }

  /// Writes the library back in the layout the engine reads.
  ///
  /// # Errors
  ///
  /// Returns an error when the library does not match its own header, or holds more objects than the grid's six bits
  /// can address.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if self.objects.len() as u64 != u64::from(self.header.object_count) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level details object count {} on write, header declares {}",
        self.objects.len(),
        self.header.object_count
      )));
    }

    if self.header.object_count > Self::MAXIMUM_OBJECTS {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level details object count {} on write, the grid addresses at most {}",
        self.header.object_count,
        Self::MAXIMUM_OBJECTS
      )));
    }

    let mut objects: ChunkWriter = ChunkWriter::new();

    for (index, object) in self.objects.iter().enumerate() {
      let mut entry: ChunkWriter = ChunkWriter::new();

      object.write::<T>(&mut entry)?;
      entry.flush_chunk_into::<T>(&mut objects.buffer, index as u32)?;
    }

    objects.flush_chunk_into::<T>(&mut writer.buffer, Self::OBJECTS_CHUNK_ID)?;

    let mut slots: ChunkWriter = ChunkWriter::new();

    slots.write_all(&self.slots)?;
    slots.flush_chunk_into::<T>(&mut writer.buffer, Self::SLOTS_CHUNK_ID)?;

    let mut header: ChunkWriter = ChunkWriter::new();

    header.write_xr::<T, _>(&self.header)?;
    header.flush_chunk_into::<T>(&mut writer.buffer, Self::HEADER_CHUNK_ID)?;

    Ok(())
  }
}

impl LevelDetailsFile {
  /// Slots the grid holds.
  pub const fn get_slots_count(&self) -> u64 {
    self.header.size_x as u64 * self.header.size_z as u64
  }

  /// How much ground the grid covers, in engine units, which are metres.
  pub fn get_covered_meters(&self) -> (f32, f32) {
    (
      self.header.size_x as f32 * DETAIL_SLOT_METERS,
      self.header.size_z as f32 * DETAIL_SLOT_METERS,
    )
  }

  /// Every slot of the grid, decoded in the order the file stores them.
  pub fn iter_slots(&self) -> impl Iterator<Item = LevelDetailsSlot> + '_ {
    self
      .slots
      .as_chunks::<{ LevelDetailsSlot::SERIALIZED_SIZE }>()
      .0
      .iter()
      .map(LevelDetailsSlot::of)
  }

  /// Slots planting at least one object, which is what decides how much of a level is actually dressed.
  pub fn get_planted_slots_count(&self) -> u64 {
    self.iter_slots().filter(LevelDetailsSlot::is_planted).count() as u64
  }

  /// How many corners across the whole grid each library object is planted in, by library index.
  pub fn get_object_usage(&self) -> Vec<u64> {
    let mut usage: Vec<u64> = vec![0; self.objects.len()];

    for slot in self.iter_slots() {
      for object in slot.objects.into_iter().flatten() {
        if let Some(count) = usage.get_mut(usize::from(object)) {
          *count += 1;
        }
      }
    }

    usage
  }
}
