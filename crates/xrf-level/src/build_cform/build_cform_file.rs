use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::{format_path, to_format_size};

use crate::build_cform::build_cform_face::BuildCformFace;
use crate::build_cform::build_cform_raycast_face::BuildCformRaycastFace;
use crate::cform::level_cform_file::LevelCformHeader;

/// The collision form xrLC leaves for the AI and detail light compilers, `build.cform`
/// (`utils/xrLC/xrBuildRapidModel.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildCformFile {
  /// The header's version, `CFORM_CURRENT_VERSION`, which both compilers reading the file assert.
  pub version: u32,
  /// The compiled scene's box, which xrAI takes as the level's.
  pub aabb_min: Vector3d<f32>,
  pub aabb_max: Vector3d<f32>,
  pub vertices: Vec<Vector3d<f32>>,
  pub faces: Vec<BuildCformFace>,
  /// One per face, at the face's index.
  pub raycast_faces: Vec<BuildCformRaycastFace>,
}

impl BuildCformFile {
  /// The file the compiler's collision form is left as, in a level's directory.
  pub const FILE_NAME: &'static str = "build.cform";

  /// `CFORM_CURRENT_VERSION`, the only one xrAI and the detail light compiler accept.
  pub const CURRENT_VERSION: u32 = 4;

  pub const GEOMETRY_CHUNK_ID: u32 = 0;

  pub const RAYCAST_FACES_CHUNK_ID: u32 = 1;

  /// Bytes one vertex occupies.
  pub const VERTEX_SIZE: u64 = 12;

  /// Reads a compiler collision form from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a compiler collision form this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Build collision form was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a compiler collision form from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a compiler collision form this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a compiler collision form this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is absent or unknown, the version is not the current one, the geometry is not
  /// exactly what its header counts, or the form is not consistent (see [`Self::write`]).
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    if let Some(chunk) = chunks
      .iter()
      .find(|chunk| chunk.id != Self::GEOMETRY_CHUNK_ID && chunk.id != Self::RAYCAST_FACES_CHUNK_ID)
    {
      return Err(XrfError::new_invalid_error(format!(
        "Build collision form carries chunk {}, which the compilers never write",
        chunk.id
      )));
    }

    let mut geometry: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::GEOMETRY_CHUNK_ID)?;
    let header: LevelCformHeader = geometry.read_xr::<T, _>()?;

    if header.version != Self::CURRENT_VERSION {
      return Err(XrfError::new_invalid_error(format!(
        "Build collision form version {} is not {}, the one the compilers read",
        header.version,
        Self::CURRENT_VERSION
      )));
    }

    let mut vertices: Vec<Vector3d<f32>> = geometry.new_bounded_vec(
      u64::from(header.vertex_count),
      Self::VERTEX_SIZE,
      "build collision form vertices",
    )?;

    for _ in 0..header.vertex_count {
      vertices.push(geometry.read_xr::<T, _>()?);
    }

    let mut faces: Vec<BuildCformFace> = geometry.new_bounded_vec(
      u64::from(header.face_count),
      BuildCformFace::SERIALIZED_SIZE,
      "build collision form faces",
    )?;

    for _ in 0..header.face_count {
      faces.push(geometry.read_xr::<T, _>()?);
    }

    geometry.assert_read("Expect all data to be read from build collision form geometry")?;

    let file: Self = Self {
      version: header.version,
      aabb_min: header.aabb_min,
      aabb_max: header.aabb_max,
      vertices,
      faces,
      raycast_faces: ChunkReader::find_required_by_id(&chunks, Self::RAYCAST_FACES_CHUNK_ID)?.read_xr_records::<T, _>(
        BuildCformRaycastFace::SERIALIZED_SIZE,
        "build collision form raycast faces chunk",
      )?,
    };

    file.assert_consistent()?;

    Ok(file)
  }

  /// Writes the form back in the layout xrLC writes it in, the header counting what the form holds.
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a vertex the form does not hold, the raycast records do not match the faces
  /// one for one, or the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.assert_consistent()?;

    let mut geometry: ChunkWriter = ChunkWriter::new();

    LevelCformHeader {
      version: self.version,
      vertex_count: to_format_size(self.vertices.len(), "build collision form vertices")?,
      face_count: to_format_size(self.faces.len(), "build collision form faces")?,
      aabb_min: self.aabb_min.clone(),
      aabb_max: self.aabb_max.clone(),
    }
    .write::<T>(&mut geometry)?;

    for vertex in &self.vertices {
      geometry.write_xr::<T, _>(vertex)?;
    }

    for face in &self.faces {
      geometry.write_xr::<T, _>(face)?;
    }

    geometry.flush_chunk_into::<T>(&mut writer.buffer, Self::GEOMETRY_CHUNK_ID)?;

    let mut raycast_faces: ChunkWriter = ChunkWriter::new();

    for face in &self.raycast_faces {
      raycast_faces.write_xr::<T, _>(face)?;
    }

    raycast_faces.flush_chunk_into::<T>(&mut writer.buffer, Self::RAYCAST_FACES_CHUNK_ID)?;

    Ok(())
  }

  /// Refuses a face naming a vertex the form does not hold, and raycast records that are not one per face, which is
  /// how many both compilers read.
  fn assert_consistent(&self) -> XrfResult {
    if let Some(face) = self
      .faces
      .iter()
      .find(|face| face.vertices.iter().any(|index| *index as usize >= self.vertices.len()))
    {
      return Err(XrfError::new_invalid_error(format!(
        "A build collision form face names vertex {:?} of {}",
        face.vertices,
        self.vertices.len()
      )));
    }

    if self.raycast_faces.len() != self.faces.len() {
      return Err(XrfError::new_invalid_error(format!(
        "Build collision form holds {} raycast records for {} faces",
        self.raycast_faces.len(),
        self.faces.len()
      )));
    }

    Ok(())
  }
}
