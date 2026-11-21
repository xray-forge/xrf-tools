use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// The collision vertices the SDK exports beside a static visual, `OGF_COLLISION_VERTICES`: a packed run of positions,
/// counted by the chunk's size.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfCollisionVerticesChunk {
  pub vertices: Vec<Vector3d<f32>>,
}

impl OgfCollisionVerticesChunk {
  /// `OGF_COLLISION_VERTICES` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 25;
}

impl ChunkReadWrite for OgfCollisionVerticesChunk {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: reader.read_remaining_records("ogf collision vertices chunk")?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_records(&self.vertices)
  }
}
