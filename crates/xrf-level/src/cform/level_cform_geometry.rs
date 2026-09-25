use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::assert_count_fits;

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
    let remaining: u64 = reader.read_bytes_remain();

    assert_count_fits(
      u64::from(header.vertex_count),
      remaining,
      Self::VERTEX_SIZE as u64,
      "collision form vertices",
    )?;

    let vertex_bytes: Vec<u8> = reader.read_bytes(header.vertex_count as usize * Self::VERTEX_SIZE)?;

    assert_count_fits(
      u64::from(header.face_count),
      reader.read_bytes_remain(),
      LevelCformFace::SERIALIZED_SIZE as u64,
      "collision form faces",
    )?;

    let face_bytes: Vec<u8> = reader.read_bytes(header.face_count as usize * LevelCformFace::SERIALIZED_SIZE)?;

    let vertices: Vec<Vector3d<f32>> = vertex_bytes
      .as_chunks::<{ Self::VERTEX_SIZE }>()
      .0
      .iter()
      .map(|vertex| {
        Vector3d::new(
          T::read_f32(&vertex[0..4]),
          T::read_f32(&vertex[4..8]),
          T::read_f32(&vertex[8..12]),
        )
      })
      .collect();

    let faces: Vec<LevelCformFace> = face_bytes
      .as_chunks::<{ LevelCformFace::SERIALIZED_SIZE }>()
      .0
      .iter()
      .map(LevelCformFace::of)
      .collect();

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
