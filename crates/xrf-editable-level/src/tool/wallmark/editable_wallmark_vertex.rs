use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
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

  pub(crate) fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      position: reader.read_xr::<T, _>()?,
      color: reader.read_u32::<T>()?,
      u: reader.read_f32::<T>()?,
      v: reader.read_f32::<T>()?,
    })
  }

  pub(crate) fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_u32::<T>(self.color)?;
    writer.write_f32::<T>(self.u)?;
    writer.write_f32::<T>(self.v)?;

    Ok(())
  }
}
