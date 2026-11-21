use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

/// One sound occluder triangle, `SOM_poly` (`xrSound/SoundRender_Scene.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSomPolygon {
  pub vertices: [Vector3d<f32>; 3],
  /// Whether the occluder blocks from both sides, which the loader turns into a second, reversed face.
  pub is_two_sided: u32,
  /// How much sound the face absorbs, handed to the collision collector as its face flags.
  pub occlusion: f32,
}

impl LevelSomPolygon {
  /// Bytes one triangle occupies: three positions, the two-sided word and the occlusion value.
  pub const SERIALIZED_SIZE: u64 = 3 * 12 + 4 + 4;

  /// Faces this triangle contributes to the sound collision model, which is two when it occludes both ways.
  pub const fn get_faces_count(&self) -> usize {
    if self.is_two_sided == 0 { 1 } else { 2 }
  }
}

impl ChunkReadWrite for LevelSomPolygon {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      vertices: [reader.read_xr()?, reader.read_xr()?, reader.read_xr()?],
      is_two_sided: reader.read_u32()?,
      occlusion: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr(vertex)?;
    }

    writer.write_u32(self.is_two_sided)?;
    writer.write_f32(self.occlusion)?;

    Ok(())
  }
}
