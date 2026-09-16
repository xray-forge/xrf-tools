use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::generic::vector_3d::Vector3d;

/// One sound occluder triangle, `SOM_poly` (`xrSound/SoundRender_Scene.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SomPolygon {
  pub vertices: [Vector3d<f32>; 3],
  /// Whether the occluder blocks from both sides, which the loader turns into a second, reversed face.
  pub is_two_sided: u32,
  /// How much sound the face absorbs, handed to the collision collector as its face flags.
  pub occlusion: f32,
}

impl SomPolygon {
  /// Bytes one triangle occupies: three positions, the two-sided word and the occlusion value.
  pub const SERIALIZED_SIZE: u64 = 3 * 12 + 4 + 4;

  /// Faces this triangle contributes to the sound collision model, which is two when it occludes both ways.
  pub const fn get_faces_count(&self) -> usize {
    if self.is_two_sided == 0 { 1 } else { 2 }
  }
}

impl ChunkReadWrite for SomPolygon {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: [
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
        reader.read_xr::<T, _>()?,
      ],
      is_two_sided: reader.read_u32::<T>()?,
      occlusion: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    writer.write_u32::<T>(self.is_two_sided)?;
    writer.write_f32::<T>(self.occlusion)?;

    Ok(())
  }
}
