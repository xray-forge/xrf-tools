use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

use crate::level::portals::level_portal::LevelPortal;

/// Every portal of a compiled level, the `fsL_PORTALS` chunk.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPortalsChunk {
  pub portals: Vec<LevelPortal>,
}

impl LevelPortalsChunk {
  /// `fsL_PORTALS` (`xray-16/src/Common/LevelStructure.hpp`).
  pub const CHUNK_ID: u32 = 4;

  /// Reads every portal from a reader positioned at the portals chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk does not divide into whole records, which is what the engine asserts on.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      portals: reader.read_xr_records::<T, _>(LevelPortal::SERIALIZED_SIZE, "level portals chunk")?,
    })
  }
}
