use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::{format_path, to_format_size};

use crate::cform::level_cform_file::LevelCformHeader;
use crate::snd_env::level_snd_env_face::LevelSndEnvFace;

/// The sound environment mesh of a level, `level.snd_env`: which reverb environment a listener is in.
///
/// Two chunks: the environment names, which the sound renderer resolves against its environment library, and a mesh
/// laid out as a collision form - a `hdrCFORM`, the vertices, then the faces - whose faces name the environment on
/// either side (`CSoundRender_Scene::set_geometry_env`). The level editor builds the mesh from box-shaped volumes, so
/// every shipped mesh is whole boxes: 8 vertices and 12 faces each.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSndEnvFile {
  /// The environments the faces name, by position: each a section of the sound environment library.
  pub environments: Vec<String>,
  /// The mesh header's version, `CFORM_CURRENT_VERSION` as the editor writes it; the sound renderer does not check it.
  pub version: u32,
  pub aabb_min: Vector3d<f32>,
  pub aabb_max: Vector3d<f32>,
  pub vertices: Vec<Vector3d<f32>>,
  pub faces: Vec<LevelSndEnvFace>,
}

impl LevelSndEnvFile {
  /// The file a level's sound environment mesh is shipped as, in its directory.
  pub const FILE_NAME: &'static str = "level.snd_env";

  pub const NAMES_CHUNK_ID: u32 = 0;

  pub const GEOMETRY_CHUNK_ID: u32 = 1;

  /// Bytes one vertex occupies.
  pub const VERTEX_SIZE: u64 = 12;

  /// Reads a sound environment mesh from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a sound environment mesh this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level sound environment mesh was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a sound environment mesh from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a sound environment mesh this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived mesh arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound environment mesh this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is absent, the mesh is not exactly what its header counts, or a face names a vertex
  /// or an environment the file does not hold - the last of which the engine asserts on.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut names: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::NAMES_CHUNK_ID)?;
    let mut environments: Vec<String> = Vec::new();

    while names.has_data() {
      environments.push(names.read_w1251_string()?);
    }

    let mut geometry: ChunkReader<D> = ChunkReader::find_required_by_id(&chunks, Self::GEOMETRY_CHUNK_ID)?;
    let header: LevelCformHeader = geometry.read_xr::<T, _>()?;
    let mut vertices: Vec<Vector3d<f32>> = geometry.new_bounded_vec(
      u64::from(header.vertex_count),
      Self::VERTEX_SIZE,
      "sound environment vertices",
    )?;

    for _ in 0..header.vertex_count {
      vertices.push(geometry.read_xr::<T, _>()?);
    }

    let mut faces: Vec<LevelSndEnvFace> = geometry.new_bounded_vec(
      u64::from(header.face_count),
      LevelSndEnvFace::SERIALIZED_SIZE,
      "sound environment faces",
    )?;

    for _ in 0..header.face_count {
      faces.push(geometry.read_xr::<T, _>()?);
    }

    geometry.assert_read("Expect all data to be read from level sound environment geometry chunk")?;

    let file: Self = Self {
      environments,
      version: header.version,
      aabb_min: header.aabb_min,
      aabb_max: header.aabb_max,
      vertices,
      faces,
    };

    file.assert_faces_resolve()?;

    Ok(file)
  }

  /// Writes the mesh back in the layout the sound renderer reads.
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a vertex or an environment the file does not hold, or the writer refuses the
  /// bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    self.assert_faces_resolve()?;

    let mut names: ChunkWriter = ChunkWriter::new();

    for environment in &self.environments {
      names.write_w1251_string(environment)?;
    }

    names.flush_chunk_into::<T>(&mut writer.buffer, Self::NAMES_CHUNK_ID)?;

    let mut geometry: ChunkWriter = ChunkWriter::new();

    LevelCformHeader {
      version: self.version,
      vertex_count: to_format_size(self.vertices.len(), "sound environment vertices")?,
      face_count: to_format_size(self.faces.len(), "sound environment faces")?,
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

    Ok(())
  }

  /// Refuses a face naming a vertex or an environment the file does not hold.
  fn assert_faces_resolve(&self) -> XrfResult {
    for face in &self.faces {
      if let Some(vertex) = face
        .vertices
        .iter()
        .find(|index| **index as usize >= self.vertices.len())
      {
        return Err(XrfError::new_invalid_error(format!(
          "A sound environment face names vertex {vertex} of {}",
          self.vertices.len()
        )));
      }

      if let Some(environment) = [face.front, face.back]
        .into_iter()
        .find(|index| usize::from(*index) >= self.environments.len())
      {
        return Err(XrfError::new_invalid_error(format!(
          "A sound environment face names environment {environment} of {}",
          self.environments.len()
        )));
      }
    }

    Ok(())
  }
}

impl LevelSndEnvFile {
  /// The environment a face's front side is in.
  pub fn get_front_environment(&self, face: &LevelSndEnvFace) -> Option<&str> {
    self.environments.get(usize::from(face.front)).map(String::as_str)
  }

  /// The environment a face's back side is in.
  pub fn get_back_environment(&self, face: &LevelSndEnvFace) -> Option<&str> {
    self.environments.get(usize::from(face.back)).map(String::as_str)
  }
}
