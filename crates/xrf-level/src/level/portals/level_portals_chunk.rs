use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader};
use xrf_error::{XrfError, XrfResult};

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
    let size: u64 = reader.read_bytes_remain();

    if !size.is_multiple_of(LevelPortal::SERIALIZED_SIZE) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level portals chunk of {size} bytes, which does not divide into {} byte records",
        LevelPortal::SERIALIZED_SIZE
      )));
    }

    let count: u64 = size / LevelPortal::SERIALIZED_SIZE;
    let mut portals: Vec<LevelPortal> = reader.new_bounded_vec(count, LevelPortal::SERIALIZED_SIZE, "level portals")?;

    for _ in 0..count {
      portals.push(LevelPortal::read::<T, D>(reader)?);
    }

    reader.assert_read("Expect all data to be read from level portals chunk")?;

    Ok(Self { portals })
  }
}
