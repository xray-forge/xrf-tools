use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One vertex of a decal's fan, `FVF::LIT`: a position, the colour baked when it was placed, and a texture coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditableWallmarkVertex {
  pub position: Vector3d,
  pub color: u32,
  pub u: f32,
  pub v: f32,
}

impl EditableWallmarkVertex {
  /// Bytes a vertex occupies.
  pub const SERIALIZED_SIZE: u64 = 12 + 4 + 8;

  pub(crate) fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr()?,
      color: reader.read_u32()?,
      u: reader.read_f32()?,
      v: reader.read_f32()?,
    })
  }

  pub(crate) fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr(&self.position)?;
    writer.write_u32(self.color)?;
    writer.write_f32(self.u)?;
    writer.write_f32(self.v)?;

    Ok(())
  }
}
