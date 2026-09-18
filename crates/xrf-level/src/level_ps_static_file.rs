use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level_ps_static_placement::PsStaticPlacement;

/// The particle effects a level plants, `level.ps_static`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPsStaticFile {
  pub version: u32,
  pub placements: Vec<PsStaticPlacement>,
}

impl LevelPsStaticFile {
  /// The version every shipped file declares, which is also the one that added the gametype word.
  pub const CURRENT_VERSION: u32 = 1;

  /// Reads the placements from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a placement list this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level static particles were not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads the placements from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a placement list this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived list arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a placement list this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// The version chunk is recognised by its size the way the engine recognises it, for the same reason the
  /// environment modifiers are: chunk 0 is a version only where it is exactly four bytes long.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk does not hold exactly one placement.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut version: u32 = 0;
    let mut placements: Vec<PsStaticPlacement> = Vec::new();

    for (index, mut chunk) in reader.read_children()?.into_iter().enumerate() {
      if index == 0 && chunk.read_bytes_remain() == size_of::<u32>() as u64 {
        version = chunk.read_u32::<T>()?;
        continue;
      }

      placements.push(PsStaticPlacement::read::<T, D>(&mut chunk, version)?);
      chunk.assert_read("Expect all data to be read from level static particle chunk")?;
    }

    Ok(Self { version, placements })
  }

  /// Writes the placements back in the layout the engine reads.
  ///
  /// # Errors
  ///
  /// Returns an error when a placement disagrees with the file's own version about the gametype word.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u32::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, 0)?;

    for (index, placement) in self.placements.iter().enumerate() {
      if placement.game_types.is_some() != (self.version > 0) {
        return Err(XrfError::new_invalid_error(format!(
          "Level static particle {index} {} a game type word its version {} does not",
          if placement.game_types.is_some() {
            "carries"
          } else {
            "omits"
          },
          self.version
        )));
      }

      let mut entry: ChunkWriter = ChunkWriter::new();

      placement.write::<T>(&mut entry)?;
      entry.flush_chunk_into::<T>(&mut writer.buffer, index as u32 + 1)?;
    }

    Ok(())
  }
}

impl LevelPsStaticFile {
  /// Placements restricted to some multiplayer modes, which a single-player session never plays.
  pub fn get_restricted_count(&self) -> usize {
    self
      .placements
      .iter()
      .filter(|placement| !placement.is_every_game_type())
      .count()
  }
}
