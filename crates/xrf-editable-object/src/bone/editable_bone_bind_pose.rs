use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// A bone's rest pose against its parent, `BONE_CHUNK_BIND_POSE`. Kept as stored: a version 1 bone stores its first
/// two rotation components swapped, which `CBone::Load_1` swaps back on load.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableBoneBindPose {
  pub offset: Vector3d,
  pub rotation: Vector3d,
  pub length: f32,
}

impl ChunkReadWrite for EditableBoneBindPose {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      offset: reader.read_xr()?,
      rotation: reader.read_xr()?,
      length: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.offset.write(writer)?;
    self.rotation.write(writer)?;
    writer.write_f32(self.length)?;

    Ok(())
  }
}
