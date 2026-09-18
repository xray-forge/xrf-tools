use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};

/// One sector of a compiled level: the portals it is seen through, and the visual it draws.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSector {
  /// Portals of this sector, by their index in the portals chunk.
  pub portals: Vec<u16>,
  /// The visual this sector draws, by its index in the visuals run.
  pub root: u32,
}

impl LevelSector {
  /// `fsP_Portals` (`xray-16/src/Common/LevelStructure.hpp`).
  pub const PORTALS_CHUNK_ID: u32 = 1;

  /// `fsP_Root`, whose payload the engine asserts is exactly four bytes.
  pub const ROOT_CHUNK_ID: u32 = 2;

  /// Bytes one portal id occupies.
  pub const PORTAL_ID_SIZE: u64 = 2;

  /// Reads one sector from a reader positioned at its chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when either chunk is absent, the portal run does not divide into ids, or the root is not one
  /// `u32` - each of which the engine asserts on rather than tolerates.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut portals_chunk: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::PORTALS_CHUNK_ID)?;
    let size: u64 = portals_chunk.read_bytes_remain();

    if !size.is_multiple_of(Self::PORTAL_ID_SIZE) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level sector portal list of {size} bytes, which does not divide into portal ids"
      )));
    }

    let mut portals: Vec<u16> = Vec::with_capacity((size / Self::PORTAL_ID_SIZE) as usize);

    for _ in 0..size / Self::PORTAL_ID_SIZE {
      portals.push(byteorder::ReadBytesExt::read_u16::<T>(&mut portals_chunk)?);
    }

    portals_chunk.assert_read("Expect all data to be read from level sector portals chunk")?;

    let mut root_chunk: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::ROOT_CHUNK_ID)?;
    let root: u32 = byteorder::ReadBytesExt::read_u32::<T>(&mut root_chunk)?;

    root_chunk.assert_read("Expect all data to be read from level sector root chunk")?;

    Ok(Self { portals, root })
  }
}
