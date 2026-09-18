use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level_light::LevelLight;
use crate::level_lights_chunk::LevelLightsChunk;

/// The compiled light list of a level, `build.lights`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelLightsFile {
  /// Every chunk in file order, so a rewrite puts them back where they were.
  pub chunks: Vec<LevelLightsChunk>,
}

impl LevelLightsFile {
  /// Reads a compiled light list from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a light list this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level lights were not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a compiled light list from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a light list this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a light list this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when the container does not account for every byte.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut chunks: Vec<LevelLightsChunk> = Vec::new();

    for mut chunk in reader.read_children()? {
      let id: u32 = chunk.id;

      chunks.push(LevelLightsChunk::read::<T, D>(id, &mut chunk)?);
    }

    Ok(Self { chunks })
  }

  /// Writes the light list back in the order the file held it.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for chunk in &self.chunks {
      let mut payload: ChunkWriter = ChunkWriter::new();

      chunk.write::<T>(&mut payload)?;
      payload.flush_chunk_into::<T>(&mut writer.buffer, chunk.get_id())?;
    }

    Ok(())
  }
}

impl LevelLightsFile {
  /// Every light of every chunk, in file order.
  pub fn iter_lights(&self) -> impl Iterator<Item = &LevelLight> {
    self.chunks.iter().flat_map(LevelLightsChunk::get_lights)
  }

  /// Lights across the whole file.
  pub fn get_lights_count(&self) -> usize {
    self.iter_lights().count()
  }

  /// The lights the runtime actually turns into light sources, which is the point ones of `fsL_HEADER`.
  pub fn get_hemi_lights(&self) -> impl Iterator<Item = &LevelLight> {
    self
      .chunks
      .iter()
      .filter(|chunk| chunk.get_id() == LevelLightsChunk::HEMI_CHUNK_ID)
      .flat_map(LevelLightsChunk::get_lights)
      .filter(|light| light.is_point())
  }
}
