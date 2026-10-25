use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// The static lighting terms a tree is shaded with, the engine's `_5color`.
#[derive(Clone, Debug, PartialEq)]
pub struct OgfTreeColor {
  pub rgb: Vector3d,
  pub hemi: f32,
  pub sun: f32,
}

impl ChunkReadWrite for OgfTreeColor {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      rgb: reader.read_xr::<T, _>()?,
      hemi: reader.read_f32::<T>()?,
      sun: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.rgb)?;
    writer.write_f32::<T>(self.hemi)?;
    writer.write_f32::<T>(self.sun)?;

    Ok(())
  }
}
