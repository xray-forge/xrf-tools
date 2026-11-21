use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One occluder triangle, `HOM_poly` (`Layers/xrRender/HOM.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelHomPolygon {
  pub vertices: [Vector3d<f32>; 3],
  /// Packed alongside the triangle and handed to the collision collector as its face flags.
  pub flags: u32,
}

impl LevelHomPolygon {
  /// Bytes one triangle occupies: three positions and a flag word.
  pub const SERIALIZED_SIZE: u64 = 3 * 12 + 4;
}

impl ChunkReadWrite for LevelHomPolygon {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: [reader.read_xr()?, reader.read_xr()?, reader.read_xr()?],
      flags: reader.read_u32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr(vertex)?;
    }

    writer.write_u32(self.flags)?;

    Ok(())
  }
}
