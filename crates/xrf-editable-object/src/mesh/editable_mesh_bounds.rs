use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// The axis-aligned box around a mesh's points, `EMESH_CHUNK_BBOX`, as the editor last computed it.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableMeshBounds {
  pub min: Vector3d,
  pub max: Vector3d,
}

impl ChunkReadWrite for EditableMeshBounds {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      min: reader.read_xr::<T, _>()?,
      max: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.min.write::<T>(writer)?;
    self.max.write::<T>(writer)?;

    Ok(())
  }
}
