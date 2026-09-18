use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::{format_path, to_format_size};

use crate::details::detail_vertex::DetailVertex;

/// One detail object, `CDetail` (`Layers/xrRender/DetailModel.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailModel {
  /// The blender the object draws through, which names a definition in `shaders.xr` rather than a file.
  pub shader: String,
  /// The texture the object draws with, which does name a file.
  pub texture: String,
  pub flags: u32,
  pub min_scale: f32,
  pub max_scale: f32,
  pub vertices: Vec<DetailVertex>,
  /// Triangle list into [`Self::vertices`], three at a time.
  pub indices: Vec<u16>,
}

impl DetailModel {
  /// The bit that turns the vertex shader's wind sway off, `DO_NO_WAVING` (`Layers/xrRender/DetailFormat.h`).
  pub const NO_WAVING: u32 = 0x0001;

  /// Reads a standalone detail model from a path, which is what a `.dm` is.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a detail model this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Detail model file was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a standalone detail model from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a detail model this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_whole::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads a standalone detail model from bytes already in hand, which is how an archived one arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a detail model this reads, or do not end where its mesh does.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_whole::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads a model that is a whole file rather than one entry of a library, so nothing may follow it.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a detail model this reads, or do not end where its mesh does.
  fn read_whole<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let model: Self = Self::read::<T, D>(reader)?;

    reader.assert_read("Expect all data to be read from detail model file")?;

    Ok(model)
  }
}

impl DetailModel {
  /// Whether the renderer sways this object in the wind, which is the flag being clear rather than set.
  pub const fn is_waving(&self) -> bool {
    self.flags & Self::NO_WAVING == 0
  }

  /// Triangles the mesh draws.
  pub const fn get_triangles_count(&self) -> usize {
    self.indices.len() / 3
  }

  /// The box the mesh occupies, or `None` for a model carrying no vertices.
  ///
  /// In engine units before the slot's own scale applies, so this is the object as authored rather than as planted.
  pub fn get_bounds(&self) -> Option<(Vector3d<f32>, Vector3d<f32>)> {
    let first: &DetailVertex = self.vertices.first()?;

    let mut minimum: Vector3d<f32> = first.position.clone();
    let mut maximum: Vector3d<f32> = first.position.clone();

    for vertex in &self.vertices {
      minimum.x = minimum.x.min(vertex.position.x);
      minimum.y = minimum.y.min(vertex.position.y);
      minimum.z = minimum.z.min(vertex.position.z);
      maximum.x = maximum.x.max(vertex.position.x);
      maximum.y = maximum.y.max(vertex.position.y);
      maximum.z = maximum.z.max(vertex.position.z);
    }

    Some((minimum, maximum))
  }
}

impl ChunkReadWrite for DetailModel {
  /// Reads a detail model, `CDetail::Load`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the mesh, or the index count is not whole triangles - which the
  /// engine asserts on rather than tolerating.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let shader: String = reader.read_w1251_string()?;
    let texture: String = reader.read_w1251_string()?;
    let flags: u32 = reader.read_u32::<T>()?;
    let min_scale: f32 = reader.read_f32::<T>()?;
    let max_scale: f32 = reader.read_f32::<T>()?;
    let vertices_count: u32 = reader.read_u32::<T>()?;
    let indices_count: u32 = reader.read_u32::<T>()?;

    if !indices_count.is_multiple_of(3) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected detail model index count {indices_count}, expected whole triangles"
      )));
    }

    let mut vertices: Vec<DetailVertex> =
      reader.new_bounded_vec(vertices_count.into(), DetailVertex::SERIALIZED_SIZE, "detail vertices")?;

    for _ in 0..vertices_count {
      vertices.push(reader.read_xr::<T, _>()?);
    }

    let mut indices: Vec<u16> =
      reader.new_bounded_vec(indices_count.into(), size_of::<u16>() as u64, "detail indices")?;

    for _ in 0..indices_count {
      indices.push(reader.read_u16::<T>()?);
    }

    Ok(Self {
      shader,
      texture,
      flags,
      min_scale,
      max_scale,
      vertices,
      indices,
    })
  }

  /// Writes a detail model back.
  ///
  /// # Errors
  ///
  /// Returns an error when either count exceeds what the format's `u32` holds, or the indices are not whole
  /// triangles.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    if !self.indices.len().is_multiple_of(3) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected detail model index count {} on write, expected whole triangles",
        self.indices.len()
      )));
    }

    writer.write_w1251_string(&self.shader)?;
    writer.write_w1251_string(&self.texture)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_f32::<T>(self.min_scale)?;
    writer.write_f32::<T>(self.max_scale)?;
    writer.write_u32::<T>(to_format_size(self.vertices.len(), "detail vertices")?)?;
    writer.write_u32::<T>(to_format_size(self.indices.len(), "detail indices")?)?;

    for vertex in &self.vertices {
      writer.write_xr::<T, _>(vertex)?;
    }

    for index in &self.indices {
      writer.write_u16::<T>(*index)?;
    }

    Ok(())
  }
}
