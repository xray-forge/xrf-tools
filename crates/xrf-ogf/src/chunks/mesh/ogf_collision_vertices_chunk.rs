use byteorder::ByteOrder;
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

  /// Bytes one vertex occupies.
  pub const VERTEX_SIZE: u64 = 12;
}

impl ChunkReadWrite for OgfCollisionVerticesChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: reader.read_xr_records::<T, _>(Self::VERTEX_SIZE, "ogf collision vertices chunk")?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    Ok(())
  }
}
