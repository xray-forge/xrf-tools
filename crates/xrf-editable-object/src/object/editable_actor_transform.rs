use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// Where the Actor Editor holds a dynamic object while it previews it, `EOBJ_CHUNK_ACTORTRANSFORM`, which it writes
/// for a dynamic object alone. The rotation is in radians, in the component order of the editor's own vector.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableActorTransform {
  pub position: Vector3d,
  pub rotation: Vector3d,
}

impl ChunkReadWrite for EditableActorTransform {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      rotation: reader.read_xr::<T, _>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.position.write::<T>(writer)?;
    self.rotation.write::<T>(writer)?;

    Ok(())
  }
}
