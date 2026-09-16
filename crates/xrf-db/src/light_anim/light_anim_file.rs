use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_optional_chunk_by_id, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::light_anim::light_anim_item::LightAnimItem;

/// The colour animation library, `lanims.xr`.
///
/// `ELightAnimLibrary::Load` (`xrEngine/LightAnimLibrary.cpp`) reads it. Anything that pulses - a lamp, a glow, a
/// campfire, a particle effect - names an animation here rather than carrying its own colours.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimFile {
  /// Version 0 stores its colours as BGR; the engine swaps them on load and writes 1 back.
  pub version: u16,
  pub items: Vec<LightAnimItem>,
}

impl LightAnimFile {
  pub const VERSION_CHUNK_ID: u32 = 0x0000;
  pub const ITEMS_CHUNK_ID: u32 = 0x0001;

  /// The version below which colours are stored channel-swapped.
  pub const BGR_VERSION: u16 = 0;

  /// Reads a colour animation library from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a colour animation library this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Light animation library was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a colour animation library from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a colour animation library this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived library arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a colour animation library this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when an animation cannot be read. A file carrying no version chunk reads as version 0, which
  /// is what `ELightAnimLibrary::Load` does with one.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      version: match find_optional_chunk_by_id(&chunks, Self::VERSION_CHUNK_ID) {
        Some(mut chunk) => chunk.read_u16::<T>()?,
        None => Self::BGR_VERSION,
      },
      items: Self::read_items::<T, D>(&mut find_required_chunk_by_id(&chunks, Self::ITEMS_CHUNK_ID)?)?,
    })
  }

  /// Writes the library back in the chunk order `ELightAnimLibrary::Load` reads it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u16::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    let mut items: ChunkWriter = ChunkWriter::new();

    for (index, item) in self.items.iter().enumerate() {
      let mut chunk: ChunkWriter = ChunkWriter::new();

      item.write::<T>(&mut chunk)?;
      chunk.flush_chunk_into::<T>(&mut items.buffer, index as u32)?;
    }

    items.flush_chunk_into::<T>(&mut writer.buffer, Self::ITEMS_CHUNK_ID)?;

    Ok(())
  }

  /// Reads every animation, one per child chunk, in the order the library numbers them.
  fn read_items<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<LightAnimItem>> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut items: Vec<LightAnimItem> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      items.push(LightAnimItem::read::<T, D>(&mut chunk)?);
    }

    Ok(items)
  }

  /// Keys across every animation.
  pub fn get_keys_count(&self) -> usize {
    self.items.iter().map(|item| item.keys.len()).sum()
  }

  /// Whether the colours are stored channel-swapped, which the engine corrects on load.
  pub const fn is_bgr(&self) -> bool {
    self.version == Self::BGR_VERSION
  }
}
