use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::cform::level_cform_face::LevelCformFace;
use crate::cform::level_cform_file::LevelCformHeader;

/// The collision form's payload, which follows its header: `Fvector[vertcount]` then `CDB::TRI[facecount]`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCformGeometry {
  /// Every vertex, in the engine's own space.
  pub vertices: Vec<Vector3d<f32>>,
  pub faces: Vec<LevelCformFace>,
}

impl LevelCformGeometry {
  /// Bytes one vertex occupies.
  pub const VERTEX_SIZE: usize = 12;

  /// Reads the payload the header counts, from just after the header.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    header: &LevelCformHeader,
  ) -> XrfResult<Self> {
    let mut vertices: Vec<Vector3d<f32>> = reader.new_bounded_vec(
      u64::from(header.vertex_count),
      Self::VERTEX_SIZE as u64,
      "collision form vertices",
    )?;

    for _ in 0..header.vertex_count {
      vertices.push(reader.read_xr::<T, _>()?);
    }

    let mut faces: Vec<LevelCformFace> = reader.new_bounded_vec(
      u64::from(header.face_count),
      LevelCformFace::SERIALIZED_SIZE as u64,
      "collision form faces",
    )?;

    for _ in 0..header.face_count {
      faces.push(reader.read_xr::<T, _>()?);
    }

    let vertex_count: u32 = header.vertex_count;

    if let Some(face) = faces
      .iter()
      .find(|face| face.vertices.iter().any(|index| *index >= vertex_count))
    {
      return Err(XrfError::new_invalid_error(format!(
        "A collision form face names vertex {:?} of {vertex_count}",
        face.vertices
      )));
    }

    Ok(Self { vertices, faces })
  }

  /// A face's three corners.
  pub fn get_triangle(&self, face: &LevelCformFace) -> [Vector3d<f32>; 3] {
    face.vertices.map(|index| self.vertices[index as usize].clone())
  }
}
