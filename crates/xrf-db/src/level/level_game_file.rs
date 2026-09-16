use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, find_optional_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::level_game_rpoint::LevelGameRPoint;
use crate::level::level_game_way::LevelGameWay;

/// The respawn points and patrol paths of a level, `level.game`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGameFile {
  pub rpoints: Vec<LevelGameRPoint>,
  pub ways: Vec<LevelGameWay>,
}

impl LevelGameFile {
  /// `RPOINT_CHUNK`, which is `POINT_BASE + ptRPoint`.
  pub const RPOINTS_CHUNK_ID: u32 = 0x2000;

  /// `WAY_PATROLPATH_CHUNK`, which is `WAY_BASE + wtPatrolPath`.
  pub const WAYS_CHUNK_ID: u32 = 0x1000;

  /// Reads a level's game data from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not level game data this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level game data was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a level's game data from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not level game data this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how archived game data arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not level game data this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// Both tables are optional: a single-player level carries paths and no respawn points, and the multiplayer ones
  /// carry both.
  ///
  /// # Errors
  ///
  /// Returns an error when a table's records do not account for their chunks.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut rpoints: Vec<LevelGameRPoint> = Vec::new();
    let mut ways: Vec<LevelGameWay> = Vec::new();

    if let Some(mut table) = find_optional_chunk_by_id(&chunks, Self::RPOINTS_CHUNK_ID) {
      for mut chunk in table.read_children()? {
        rpoints.push(LevelGameRPoint::read::<T, D>(&mut chunk)?);
        chunk.assert_read("Expect all data to be read from level respawn point chunk")?;
      }
    }

    if let Some(mut table) = find_optional_chunk_by_id(&chunks, Self::WAYS_CHUNK_ID) {
      for mut chunk in table.read_children()? {
        ways.push(LevelGameWay::read::<T, D>(&mut chunk)?);
      }
    }

    Ok(Self { rpoints, ways })
  }

  /// Writes the game data back, paths first, which is the order every shipped file uses.
  ///
  /// # Errors
  ///
  /// Returns an error when a count exceeds what the format holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut ways: ChunkWriter = ChunkWriter::new();

    for (index, way) in self.ways.iter().enumerate() {
      let mut entry: ChunkWriter = ChunkWriter::new();

      way.write::<T>(&mut entry)?;
      entry.flush_chunk_into::<T>(&mut ways.buffer, index as u32)?;
    }

    ways.flush_chunk_into::<T>(&mut writer.buffer, Self::WAYS_CHUNK_ID)?;

    let mut rpoints: ChunkWriter = ChunkWriter::new();

    for (index, rpoint) in self.rpoints.iter().enumerate() {
      let mut entry: ChunkWriter = ChunkWriter::new();

      rpoint.write::<T>(&mut entry)?;
      entry.flush_chunk_into::<T>(&mut rpoints.buffer, index as u32)?;
    }

    rpoints.flush_chunk_into::<T>(&mut writer.buffer, Self::RPOINTS_CHUNK_ID)?;

    Ok(())
  }
}

impl LevelGameFile {
  /// Nodes across every patrol path.
  pub fn get_way_points_count(&self) -> usize {
    self.ways.iter().map(|way| way.points.len()).sum()
  }

  /// Respawn points that name a spawn preset, which only item points do.
  pub fn get_profiled_rpoints_count(&self) -> usize {
    self.rpoints.iter().filter(|rpoint| !rpoint.profile.is_empty()).count()
  }
}
