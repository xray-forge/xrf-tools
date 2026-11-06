use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
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

  /// Writes a skinned child's vertices and indices as the skeleton export writes them: the vertices chunk tagged with
  /// the plain link count every vertex is laid out in, then the indices chunk, its count first.
  ///
  /// # Errors
  ///
  /// Returns an error when the vertices are not all laid out in one link count, or a count outgrows a `u32`.
  pub fn write_skinned<T: ByteOrder>(writer: &mut ChunkWriter, vertices: &[OgfVertex], indices: &[u16]) -> XrfResult {
    let links: usize = vertices.first().map_or(1, |vertex| vertex.links.len());

    if vertices.iter().any(|vertex| vertex.links.len() != links) {
      return Err(XrfError::new_invalid_error(
        "A skinned child's vertices are written in one link layout, the child's widest",
      ));
    }

    let count = |length: usize, what: &str| {
      u32::try_from(length)
        .map_err(|_| XrfError::new_invalid_error(format!("A child holds more {what} than a u32 counts")))
    };
    let mut vertices_chunk: ChunkWriter = ChunkWriter::new();

    vertices_chunk.write_u32::<T>(count(links, "links")?)?;
    vertices_chunk.write_u32::<T>(count(vertices.len(), "vertices")?)?;

    for vertex in vertices {
      vertex.write::<T>(&mut vertices_chunk)?;
    }

    let mut indices_chunk: ChunkWriter = ChunkWriter::new();

    indices_chunk.write_u32::<T>(count(indices.len(), "indices")?)?;

    for index in indices {
      indices_chunk.write_u16::<T>(*index)?;
    }

    vertices_chunk.flush_chunk_into::<T>(writer, Self::VERTICES_CHUNK_ID)?;
    indices_chunk.flush_chunk_into::<T>(writer, Self::INDICES_CHUNK_ID)?;

    Ok(())
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
  use crate::data::geometry::skin::ogf_vertex::OgfVertex;
  use crate::data::geometry::skin::ogf_vertex_link::OgfVertexLink;
  use crate::data::geometry::skin::ogf_vertex_links::OgfVertexLinks;
  use crate::data::ogf_model_type::OgfModelType;

  fn skinned(links: &[(u16, f32)], texture_u: f32) -> OgfVertex {
    OgfVertex {
      position: Vector3d::new(1.0, 2.0, 3.0),
      normal: Vector3d::Y,
      tangent: Vector3d::X,
      binormal: Vector3d::Z,
      texture_u,
      texture_v: 0.5,
      links: OgfVertexLinks::from_slice(
        &links
          .iter()
          .map(|(bone, weight)| OgfVertexLink {
            bone: *bone,
            weight: *weight,
          })
          .collect::<Vec<_>>(),
      ),
    }
  }

  fn write_then_read(vertices: &[OgfVertex], indices: &[u16]) -> XrfResult<OgfGeometry> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    OgfGeometry::write_skinned::<XRayByteOrder>(&mut writer, vertices, indices)?;

    let bytes: Vec<u8> = writer.flush_raw_into_buffer()?;
    let chunks = ChunkReader::from_bytes(&bytes)?.read_children()?;

    Ok(
      OgfGeometry::read_from_chunks::<XRayByteOrder, _>(&chunks, Some(OgfModelType::SkeletonGeomdefSt))?
        .expect("geometry chunks are present"),
    )
  }

  #[test]
  fn writes_skinned_vertices_in_each_layout_that_read_back_alike() -> XrfResult {
    // Weights a binary fraction holds exactly, since a reader derives the last from the rest.
    for vertices in [
      vec![skinned(&[(7, 1.0)], 0.25), skinned(&[(2, 1.0)], 0.75)],
      vec![skinned(&[(1, 0.75), (4, 0.25)], 0.25)],
      vec![skinned(&[(1, 0.5), (3, 0.25), (2, 0.25)], 0.25)],
    ] {
      let geometry: OgfGeometry = write_then_read(&vertices, &[0, 0, 0])?;

      assert_eq!(geometry.vertex_format, Some(vertices[0].links.len() as u32));
      assert_eq!(geometry.get_skinned_vertices(), Some(vertices.as_slice()));
      assert_eq!(geometry.indices, Some(vec![0, 0, 0]));
    }

    Ok(())
  }

  #[test]
  fn refuses_vertices_of_two_layouts_in_one_child() {
    let vertices: [OgfVertex; 2] = [skinned(&[(1, 1.0)], 0.0), skinned(&[(1, 0.5), (2, 0.5)], 0.0)];

    assert!(OgfGeometry::write_skinned::<XRayByteOrder>(&mut ChunkWriter::new(), &vertices, &[]).is_err());
  }

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
