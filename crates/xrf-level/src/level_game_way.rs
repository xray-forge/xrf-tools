use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_optional_chunk_by_id};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::level_game_way_link::LevelGameWayLink;
use crate::level_game_way_point::LevelGameWayPoint;

/// One patrol path of a level, as the editor authored it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGameWay {
  pub version: u16,
  pub name: String,
  /// `EWayType`, absent in every shipped file - the editor writes the chunk only for a path that is not the default
  /// patrol kind, and nothing across the five workspace trees is.
  pub kind: Option<u32>,
  pub points: Vec<LevelGameWayPoint>,
  pub links: Vec<LevelGameWayLink>,
}

impl LevelGameWay {
  pub const VERSION_CHUNK_ID: u32 = 0x0001;
  pub const POINTS_CHUNK_ID: u32 = 0x0002;
  pub const LINKS_CHUNK_ID: u32 = 0x0003;
  pub const TYPE_CHUNK_ID: u32 = 0x0004;
  pub const NAME_CHUNK_ID: u32 = 0x0005;

  /// Reads one path off its own chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk does not account for what it declares.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      version: match find_optional_chunk_by_id(&chunks, Self::VERSION_CHUNK_ID) {
        Some(mut chunk) => chunk.read_u16::<T>()?,
        None => 0,
      },
      name: match find_optional_chunk_by_id(&chunks, Self::NAME_CHUNK_ID) {
        Some(mut chunk) => chunk.read_w1251_string()?,
        None => String::new(),
      },
      kind: match find_optional_chunk_by_id(&chunks, Self::TYPE_CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_u32::<T>()?),
        None => None,
      },
      points: Self::read_points::<T, D>(&chunks)?,
      links: Self::read_links::<T, D>(&chunks)?,
    })
  }

  /// Writes one path back, in the chunk order the editor emits.
  ///
  /// # Errors
  ///
  /// Returns an error when a count exceeds what the format's `u16` holds.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u16::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    let mut name: ChunkWriter = ChunkWriter::new();

    name.write_w1251_string(&self.name)?;
    name.flush_chunk_into::<T>(&mut writer.buffer, Self::NAME_CHUNK_ID)?;

    if let Some(value) = self.kind {
      let mut kind: ChunkWriter = ChunkWriter::new();

      kind.write_u32::<T>(value)?;
      kind.flush_chunk_into::<T>(&mut writer.buffer, Self::TYPE_CHUNK_ID)?;
    }

    let mut points: ChunkWriter = ChunkWriter::new();

    points.write_u16::<T>(to_format_size(self.points.len(), "patrol path points")?)?;

    for point in &self.points {
      points.write_xr::<T, _>(point)?;
    }

    points.flush_chunk_into::<T>(&mut writer.buffer, Self::POINTS_CHUNK_ID)?;

    let mut links: ChunkWriter = ChunkWriter::new();

    links.write_u16::<T>(to_format_size(self.links.len(), "patrol path links")?)?;

    for link in &self.links {
      links.write_xr::<T, _>(link)?;
    }

    links.flush_chunk_into::<T>(&mut writer.buffer, Self::LINKS_CHUNK_ID)?;

    Ok(())
  }

  /// The path's nodes, or none where it declares no node chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the declared count does not fit the chunk, or the chunk ends inside a node.
  fn read_points<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Vec<LevelGameWayPoint>> {
    let Some(mut chunk) = find_optional_chunk_by_id(chunks, Self::POINTS_CHUNK_ID) else {
      return Ok(Vec::new());
    };

    let count: u16 = chunk.read_u16::<T>()?;
    let mut points: Vec<LevelGameWayPoint> =
      chunk.new_bounded_vec(count.into(), LevelGameWayPoint::FIXED_SIZE, "patrol path points")?;

    for _ in 0..count {
      points.push(chunk.read_xr::<T, _>()?);
    }

    chunk.assert_read("Expect all data to be read from patrol path points chunk")?;

    Ok(points)
  }

  /// The path's edges, or none where it declares no link chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the declared count does not fit the chunk, or the chunk ends inside a link.
  fn read_links<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Vec<LevelGameWayLink>> {
    let Some(mut chunk) = find_optional_chunk_by_id(chunks, Self::LINKS_CHUNK_ID) else {
      return Ok(Vec::new());
    };

    let count: u16 = chunk.read_u16::<T>()?;
    let mut links: Vec<LevelGameWayLink> =
      chunk.new_bounded_vec(count.into(), LevelGameWayLink::SERIALIZED_SIZE, "patrol path links")?;

    for _ in 0..count {
      links.push(chunk.read_xr::<T, _>()?);
    }

    chunk.assert_read("Expect all data to be read from patrol path links chunk")?;

    Ok(links)
  }
}
