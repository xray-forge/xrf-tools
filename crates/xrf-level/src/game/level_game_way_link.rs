use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

/// One edge between two nodes of a patrol path, by their positions in the path's own node list.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelGameWayLink {
  pub from: u16,
  pub to: u16,
  pub probability: f32,
}

impl LevelGameWayLink {
  /// Bytes a link occupies.
  pub const SERIALIZED_SIZE: u64 = 2 + 2 + 4;
}

impl ChunkReadWrite for LevelGameWayLink {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      from: reader.read_u16()?,
      to: reader.read_u16()?,
      probability: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16(self.from)?;
    writer.write_u16(self.to)?;
    writer.write_f32(self.probability)?;

    Ok(())
  }
}
