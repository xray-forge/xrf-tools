use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      rgb: reader.read_xr()?,
      hemi: reader.read_f32()?,
      sun: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr(&self.rgb)?;
    writer.write_f32(self.hemi)?;
    writer.write_f32(self.sun)?;

    Ok(())
  }
}
