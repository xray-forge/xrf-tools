use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::XrfResult;

use crate::level::glows::level_glow::LevelGlow;

/// Every glow of a compiled level, the `fsL_GLOWS` chunk: a packed run of records, counted by its size.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGlowsChunk {
  pub glows: Vec<LevelGlow>,
}

impl LevelGlowsChunk {
  /// `fsL_GLOWS` (`xray-16/src/Common/LevelStructure.hpp`).
  pub const CHUNK_ID: u32 = 7;

  /// Reads every glow from a reader positioned at the glows chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk does not divide into whole records, which is what the R1 loader asserts on.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      glows: reader.read_xr_records::<T, _>(LevelGlow::SERIALIZED_SIZE, "level glows chunk")?,
    })
  }
}
