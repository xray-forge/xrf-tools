use byteorder::{ByteOrder, ReadBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};

use crate::data::geometry::ogf_fvf::OgfFvf;
use crate::data::geometry::ogf_static_vertex::OgfStaticVertex;
use crate::data::geometry::ogf_vertex_data::OgfVertexData;
use crate::data::geometry::ogf_vertices::OgfVertices;
use crate::data::geometry::skin::ogf_skin_layout::OgfSkinLayout;
use crate::data::geometry::skin::ogf_vertex::OgfVertex;
use crate::data::ogf_model_type::OgfModelType;

/// Geometry embedded directly in an OGF visual.
#[derive(Debug)]
pub struct OgfGeometry {
  pub vertex_count: Option<u32>,
  pub indices: Option<Vec<u16>>,
  pub skin_bone_indices: Vec<u16>,
  /// Stored vertex format tag, kept so the data can be written back in the form it arrived in.
  pub vertex_format: Option<u32>,
  /// Fully decoded vertices, absent for a format whose layout is not known.
  pub vertices: Option<OgfVertexData>,
}

impl OgfGeometry {
  pub const VERTICES_CHUNK_ID: u32 = 3;
  pub const INDICES_CHUNK_ID: u32 = 4;

  /// The skinned vertices, for a skinned child visual whose format is known.
  pub fn get_skinned_vertices(&self) -> Option<&[OgfVertex]> {
    match &self.vertices {
      Some(OgfVertexData::Skinned(vertices)) => Some(vertices),
      _ => None,
    }
  }

  /// The static vertices, for any other visual whose flexible vertex format is known.
  pub fn get_static_vertices(&self) -> Option<&[OgfStaticVertex]> {
    match &self.vertices {
      Some(OgfVertexData::Static(vertices)) => Some(vertices),
      _ => None,
    }
  }

  /// Reads the embedded vertices and indices of a visual of `model_type`, which decides how its vertex format tag is
  /// read: a skinned child's as a skin layout, any other visual's as a flexible vertex format, as the engine does.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
    model_type: Option<OgfModelType>,
  ) -> XrfResult<Option<Self>> {
    let vertices: Option<OgfVertices> = match chunks.iter().find(|chunk| chunk.id == Self::VERTICES_CHUNK_ID) {
      Some(chunk) => Some(Self::read_vertices::<T, D>(&mut chunk.clone(), model_type)?),
      None => None,
    };

    let indices: Option<Vec<u16>> = match chunks.iter().find(|chunk| chunk.id == Self::INDICES_CHUNK_ID) {
      Some(chunk) => Some(Self::read_indices::<T, D>(&mut chunk.clone())?),
      None => None,
    };

    match (vertices, indices) {
      (None, None) => Ok(None),
      (vertices, indices) => Ok(Some(Self {
        vertex_count: vertices.as_ref().map(|it| it.count),
        indices,
        skin_bone_indices: vertices
          .as_ref()
          .map(OgfVertices::collect_bone_indices)
          .unwrap_or_default(),
        vertex_format: vertices.as_ref().map(|it| it.format),
        vertices: vertices.and_then(|it| it.data),
      })),
    }
  }

  fn read_vertices<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    model_type: Option<OgfModelType>,
  ) -> XrfResult<OgfVertices> {
    let vertex_format: u32 = reader.read_u32::<T>()?;
    let vertex_count: u32 = reader.read_u32::<T>()?;

    // Decoded where the payload lies: a visual's vertices are most of its bytes, and only the decoded form is kept.
    let data: Option<OgfVertexData> = reader.read_remaining_with(|vertices| {
      if model_type.is_some_and(OgfModelType::is_skinned_geometry) {
        Self::decode_skinned_vertices::<T>(vertices, vertex_format, vertex_count)
      } else {
        Self::decode_static_vertices::<T>(vertices, vertex_format, vertex_count)
      }
    })??;

    Ok(OgfVertices {
      format: vertex_format,
      count: vertex_count,
      data,
    })
  }

  /// The vertices of a skinned payload, or `None` for a skin layout the format tag does not name.
  fn decode_skinned_vertices<T: ByteOrder>(
    vertices: &[u8],
    vertex_format: u32,
    vertex_count: u32,
  ) -> XrfResult<Option<OgfVertexData>> {
    let Some(layout) = OgfSkinLayout::of(vertex_format) else {
      return Ok(None);
    };

    Ok(Some(OgfVertexData::Skinned(
      Self::sized_vertices(vertices, vertex_count, layout.vertex_size, "skin")?
        .map(|vertex| OgfVertex::read_from_slice::<T>(vertex, layout.links))
        .collect(),
    )))
  }

  /// The vertices of a static payload, or `None` for a flexible vertex format this does not decode.
  fn decode_static_vertices<T: ByteOrder>(
    vertices: &[u8],
    vertex_format: u32,
    vertex_count: u32,
  ) -> XrfResult<Option<OgfVertexData>> {
    let Some(format) = OgfFvf::from_raw(vertex_format) else {
      return Ok(None);
    };

    Ok(Some(OgfVertexData::Static(
      Self::sized_vertices(vertices, vertex_count, format.get_stride(), "static")?
        .map(|vertex| OgfStaticVertex::read_from_slice::<T>(vertex, &format))
        .collect(),
    )))
  }

  /// The vertex records of a payload that is exactly `count` records of `vertex_size`, which the engine assumes.
  fn sized_vertices<'a>(
    vertices: &'a [u8],
    count: u32,
    vertex_size: usize,
    family: &str,
  ) -> XrfResult<std::slice::ChunksExact<'a, u8>> {
    let expected_size: usize = (count as usize)
      .checked_mul(vertex_size)
      .ok_or_else(|| XrfError::new_invalid_error(format!("OGF {family} vertex data size overflows")))?;

    if vertices.len() != expected_size {
      return Err(XrfError::new_invalid_error(format!(
        "Invalid OGF {family} vertex data size: expected {expected_size} bytes for {count} vertices, got {}",
        vertices.len()
      )));
    }

    Ok(vertices.chunks_exact(vertex_size))
  }

  fn read_indices<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<u16>> {
    let indices: Vec<u16> = reader.read_u16_vector::<T>()?;
    reader.assert_read("Expect all data to be read from ogf indices")?;

    Ok(indices)
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use byteorder::WriteBytesExt;
  use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use super::OgfGeometry;
  use crate::data::geometry::ogf_static_vertex::OgfStaticVertex;
  use crate::data::ogf_model_type::OgfModelType;

  #[test]
  fn test_read() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "read.chunk");
    let contents: Vec<u8> = geometry_contents()?;
    let mut file = GeneratedResources::overwrite(&filename)?;

    file.write_all(&contents)?;
    file.flush()?;

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;
    assert_eq!(file.bytes_remaining(), contents.len());

    let chunks = ChunkReader::from_slice(file)?.read_children()?;
    let geometry: OgfGeometry =
      OgfGeometry::read_from_chunks::<XRayByteOrder, _>(&chunks, Some(OgfModelType::SkeletonGeomdefSt))?
        .expect("geometry chunks are present");

    assert_eq!(geometry.vertex_count, Some(2));
    assert_eq!(geometry.indices, Some(vec![0, 1, 0]));
    assert_eq!(geometry.skin_bone_indices, vec![1, 3, 2, 4]);
    assert_eq!(geometry.get_skinned_vertices().map(<[_]>::len), Some(2));

    Ok(())
  }

  #[test]
  fn reads_a_static_visuals_tag_as_its_flexible_vertex_format() -> XrfResult {
    // The SDK's static export, `D3DFVF_XYZ | D3DFVF_NORMAL | D3DFVF_TEX1`: position, normal and one uv pair.
    let mut vertices: ChunkWriter = ChunkWriter::new();

    vertices.write_u32::<XRayByteOrder>(0x112)?;
    vertices.write_u32::<XRayByteOrder>(1)?;

    for value in [1.0f32, 2.0, 3.0, 0.0, 1.0, 0.0, 0.25, 0.75] {
      vertices.write_f32::<XRayByteOrder>(value)?;
    }

    let bytes: Vec<u8> = vertices.flush_chunk_into_buffer::<XRayByteOrder>(OgfGeometry::VERTICES_CHUNK_ID)?;
    let chunks = ChunkReader::from_bytes(&bytes)?.read_children()?;
    let geometry: OgfGeometry = OgfGeometry::read_from_chunks::<XRayByteOrder, _>(&chunks, Some(OgfModelType::Normal))?
      .expect("geometry chunks are present");
    let vertex: &OgfStaticVertex = &geometry.get_static_vertices().expect("static vertices are decoded")[0];

    assert_eq!(vertex.position, Vector3d::new(1.0, 2.0, 3.0));
    assert_eq!(vertex.normal, Some(Vector3d::new(0.0, 1.0, 0.0)));
    assert_eq!(vertex.texture_coordinates, vec![vec![0.25, 0.75]]);
    assert_eq!((vertex.diffuse, vertex.specular, vertex.point_size), (None, None, None));
    assert!(geometry.skin_bone_indices.is_empty());

    Ok(())
  }

  #[test]
  fn a_tag_the_visuals_family_has_no_layout_for_keeps_its_tag_alone() -> XrfResult {
    // The legacy one-link skin tag, read off a static visual, is a flexible vertex format with no position: which family
    // a tag belongs to is the visual's kind, never the tag, so it is kept undecoded rather than read as a skin.
    let mut vertices: ChunkWriter = ChunkWriter::new();

    vertices.write_u32::<XRayByteOrder>(0x12071980)?;
    vertices.write_u32::<XRayByteOrder>(0)?;

    let bytes: Vec<u8> = vertices.flush_chunk_into_buffer::<XRayByteOrder>(OgfGeometry::VERTICES_CHUNK_ID)?;
    let chunks = ChunkReader::from_bytes(&bytes)?.read_children()?;
    let geometry: OgfGeometry = OgfGeometry::read_from_chunks::<XRayByteOrder, _>(&chunks, Some(OgfModelType::Normal))?
      .expect("geometry chunks are present");

    assert_eq!(geometry.vertex_format, Some(0x12071980));
    assert_eq!(geometry.vertices, None);

    Ok(())
  }

  fn geometry_contents() -> XrfResult<Vec<u8>> {
    let mut vertices: ChunkWriter = ChunkWriter::new();

    vertices.write_u32::<XRayByteOrder>(2)?;
    vertices.write_u32::<XRayByteOrder>(2)?;
    vertices.write_u16::<XRayByteOrder>(1)?;
    vertices.write_u16::<XRayByteOrder>(3)?;
    vertices.write_all(&[0; 60])?;
    vertices.write_u16::<XRayByteOrder>(2)?;
    vertices.write_u16::<XRayByteOrder>(4)?;
    vertices.write_all(&[0; 60])?;

    let mut indices: ChunkWriter = ChunkWriter::new();

    indices.write_u32::<XRayByteOrder>(3)?;
    indices.write_u16::<XRayByteOrder>(0)?;
    indices.write_u16::<XRayByteOrder>(1)?;
    indices.write_u16::<XRayByteOrder>(0)?;

    let mut source: ChunkWriter = ChunkWriter::new();

    source.write_all(&vertices.flush_chunk_into_buffer::<XRayByteOrder>(OgfGeometry::VERTICES_CHUNK_ID)?)?;
    source.write_all(&indices.flush_chunk_into_buffer::<XRayByteOrder>(OgfGeometry::INDICES_CHUNK_ID)?)?;

    source.flush_raw_into_buffer()
  }
}
