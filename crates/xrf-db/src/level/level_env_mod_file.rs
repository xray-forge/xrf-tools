use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::level_env_modifier::EnvModifier;

/// The local weather overrides of a level, `level.env_mod`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelEnvModFile {
  pub version: u32,
  pub modifiers: Vec<EnvModifier>,
}

impl LevelEnvModFile {
  /// What `CEnvironment::mods_load` assumes when the file declares nothing.
  pub const DEFAULT_VERSION: u32 = 0x0015;

  pub const VERSION_CHUNK_ID: u32 = 0;

  /// Reads the modifiers from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a modifier list this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level environment modifiers were not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads the modifiers from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a modifier list this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a modifier list this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is not the size its version gives a modifier.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: u32 = Self::DEFAULT_VERSION;
    let mut modifiers: Vec<EnvModifier> = Vec::new();

    for (index, mut chunk) in reader.read_children()?.into_iter().enumerate() {
      if index == 0 && chunk.read_bytes_remain() == size_of::<u32>() as u64 {
        version = chunk.read_u32::<T>()?;
        continue;
      }

      modifiers.push(EnvModifier::read::<T, D>(&mut chunk, version)?);
      chunk.assert_read("Expect all data to be read from level environment modifier chunk")?;
    }

    Ok(Self { version, modifiers })
  }

  /// Writes the modifiers back in the layout the engine reads.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u32::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    for (index, modifier) in self.modifiers.iter().enumerate() {
      let mut entry: ChunkWriter = ChunkWriter::new();

      modifier.write::<T>(&mut entry)?;
      entry.flush_chunk_into::<T>(&mut writer.buffer, index as u32 + 1)?;
    }

    Ok(())
  }
}
