use byteorder::ByteOrder;
use glam::Vec3;
use serde::Serialize;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_geometry::{BvhTracer, Triangle};
use xrf_math::Vector3d;

use crate::cform::level_cform_face::LevelCformFace;
use crate::cform::level_cform_file::LevelCformHeader;

/// The collision form's payload, which follows its header: `Fvector[vertcount]` then `CDB::TRI[facecount]`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCformGeometry {
  /// Every vertex, in the engine's own space.
  vertices: Vec<Vector3d<f32>>,
  /// Every face, each naming only vertices the form holds.
  faces: Vec<LevelCformFace>,
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

    Self::new(vertices, faces)
  }

  /// A form of the given vertices and the faces between them.
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a vertex the form does not hold.
  pub fn new(vertices: Vec<Vector3d<f32>>, faces: Vec<LevelCformFace>) -> XrfResult<Self> {
    if let Some(face) = faces
      .iter()
      .find(|face| face.vertices.iter().any(|index| *index as usize >= vertices.len()))
    {
      return Err(XrfError::new_invalid_error(format!(
        "A collision form face names vertex {:?} of {}",
        face.vertices,
        vertices.len()
      )));
    }

    Ok(Self { vertices, faces })
  }

  /// Writes the payload, every vertex then every face, as [`Self::read_from_chunk`] reads it.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer fails.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    for face in &self.faces {
      writer.write_xr::<T, _>(face)?;
    }

    Ok(())
  }

  /// Every vertex, in the engine's own space.
  pub fn get_vertices(&self) -> &[Vector3d<f32>] {
    &self.vertices
  }

  /// Every face.
  pub fn get_faces(&self) -> &[LevelCformFace] {
    &self.faces
  }

  /// A face's three corners.
  pub fn get_triangle(&self, face: &LevelCformFace) -> [Vector3d<f32>; 3] {
    face.vertices.map(|index| self.vertices[index as usize])
  }

  /// The form's faces in a tracer for the engine's ray and box queries, each named by its index here.
  pub fn build_tracer(&self) -> BvhTracer {
    BvhTracer::of_triangles(
      self
        .faces
        .iter()
        .map(|face| Triangle::new(face.vertices.map(|index| Vec3::from(&self.vertices[index as usize]))))
        .collect(),
    )
  }
}
