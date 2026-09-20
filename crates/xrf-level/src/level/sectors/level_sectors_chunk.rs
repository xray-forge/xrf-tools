use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

use crate::level::sectors::level_sector::LevelSector;

/// Every sector of a compiled level, the `fsL_SECTORS` chunk.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSectorsChunk {
  pub sectors: Vec<LevelSector>,
}

impl LevelSectorsChunk {
  /// `fsL_SECTORS` (`xray-16/src/Common/LevelStructure.hpp`).
  pub const CHUNK_ID: u32 = 8;

  /// Reads every sector from a reader positioned at the sectors chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when any sector cannot be read. One unreadable sector fails the chunk rather than being
  /// skipped, because a gap would renumber every sector after it and the number is what portals name.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut sectors: Vec<LevelSector> = Vec::with_capacity(chunks.len());

    for mut chunk in chunks {
      sectors.push(LevelSector::read_from_chunk::<T, D>(&mut chunk)?);
    }

    Ok(Self { sectors })
  }
}
