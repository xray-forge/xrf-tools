use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One occluder triangle, `HOM_poly` (`Layers/xrRender/HOM.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HomPolygon {
  pub vertices: [Vector3d<f32>; 3],
  /// Packed alongside the triangle and handed to the collision collector as its face flags.
  pub flags: u32,
}

impl HomPolygon {
  /// Bytes one triangle occupies: three positions and a flag word.
  pub const SERIALIZED_SIZE: u64 = 3 * 12 + 4;
}

impl ChunkReadWrite for HomPolygon {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
      flags: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    writer.write_u32::<T>(self.flags)?;

    Ok(())
  }
}
