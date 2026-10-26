use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::ogf_collision_face::OgfCollisionFace;

/// The collision triangles the SDK exports beside a static visual, `OGF_COLLISION_INDICES`: a packed run of faces into
/// [`crate::OgfCollisionVerticesChunk`], counted by the chunk's size. Like the vertices, no engine loader reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfCollisionIndicesChunk {
  pub faces: Vec<OgfCollisionFace>,
}

impl OgfCollisionIndicesChunk {
  /// `OGF_COLLISION_INDICES` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 26;
}

impl ChunkReadWrite for OgfCollisionIndicesChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      faces: reader.read_xr_records::<T, _>(OgfCollisionFace::SERIALIZED_SIZE, "ogf collision indices chunk")?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for face in &self.faces {
      writer.write_xr::<T, _>(face)?;
    }

    Ok(())
  }
}
