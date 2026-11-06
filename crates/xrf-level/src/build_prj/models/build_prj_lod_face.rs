use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::{Vector2d, Vector3d};

/// One quad of an impostor, `b_lod_face` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjLodFace {
  pub vertices: [Vector3d<f32>; 4],
  pub texture_coordinates: [Vector2d; 4],
}

impl BuildPrjLodFace {
  /// Bytes one quad occupies.
  pub const SERIALIZED_SIZE: u64 = 4 * 12 + 4 * 8;
}

impl ChunkReadWrite for BuildPrjLodFace {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut vertices: [Vector3d<f32>; 4] = [const { Vector3d::ZERO }; 4];

    for vertex in &mut vertices {
      *vertex = reader.read_xr::<T, _>()?;
    }

    let mut texture_coordinates: [Vector2d; 4] = [Vector2d::ZERO; 4];

    for coordinate in &mut texture_coordinates {
      *coordinate = reader.read_xr::<T, _>()?;
    }

    Ok(Self {
      vertices,
      texture_coordinates,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    for coordinate in &self.texture_coordinates {
      writer.write_xr::<T, _>(coordinate)?;
    }

    Ok(())
  }
}
