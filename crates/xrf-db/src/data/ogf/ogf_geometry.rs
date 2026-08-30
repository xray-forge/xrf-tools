use byteorder::{ByteOrder, ReadBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};

use crate::data::ogf::ogf_vertex::OgfVertex;
use crate::data::ogf::ogf_vertices::OgfVertices;

/// Geometry embedded directly in an OGF visual.
///
/// Geometry can instead be stored in shared vertex or index containers. In
/// that case the corresponding field remains absent because the container is
/// outside this OGF file.
#[derive(Debug)]
pub struct OgfGeometry {
  pub vertex_count: Option<u32>,
  pub indices: Option<Vec<u16>>,
  pub skin_bone_indices: Vec<u16>,
  /// Stored vertex format tag, kept so the data can be written back in the form it arrived in.
  pub vertex_format: Option<u32>,
  /// Fully decoded vertices, absent for a format whose layout is not known.
  pub vertices: Option<Vec<OgfVertex>>,
}

impl OgfGeometry {
  pub const VERTICES_CHUNK_ID: u32 = 3;
  pub const INDICES_CHUNK_ID: u32 = 4;

  const VERTEX_FORMAT_1_LINK: u32 = 0x12071980;
  const VERTEX_FORMAT_2_LINK: u32 = 2 * Self::VERTEX_FORMAT_1_LINK;
  const VERTEX_FORMAT_3_LINK: u32 = 4 * Self::VERTEX_FORMAT_1_LINK;
  const VERTEX_FORMAT_4_LINK: u32 = 5 * Self::VERTEX_FORMAT_1_LINK;

  const VERTEX_SIZE_1_LINK: usize = 60;
  const VERTEX_SIZE_2_LINK: usize = 64;
  const VERTEX_SIZE_3_LINK: usize = 70;
  const VERTEX_SIZE_4_LINK: usize = 76;

  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Option<Self>> {
    let vertices: Option<OgfVertices> = match chunks.iter().find(|chunk| chunk.id == Self::VERTICES_CHUNK_ID) {
      Some(chunk) => Some(Self::read_vertices::<T, D>(&mut chunk.clone())?),
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
        vertices: vertices.and_then(|it| it.vertices),
      })),
    }
  }

  fn read_vertices<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<OgfVertices> {
    let vertex_format: u32 = reader.read_u32::<T>()?;
    let vertex_count: u32 = reader.read_u32::<T>()?;
    let vertices: Vec<u8> = reader.read_remaining()?;

    let Some((vertex_size, bone_indices_per_vertex)) = Self::skin_vertex_layout(vertex_format) else {
      return Ok(OgfVertices {
        format: vertex_format,
        count: vertex_count,
        vertices: None,
      });
    };

    let expected_size: usize = (vertex_count as usize)
      .checked_mul(vertex_size)
      .ok_or_else(|| XrfError::new_invalid_error("OGF skin vertex data size overflows"))?;

    if vertices.len() != expected_size {
      return Err(XrfError::new_invalid_error(format!(
        "Invalid OGF skin vertex data size: expected {expected_size} bytes for {vertex_count} vertices, got {}",
        vertices.len()
      )));
    }

    let mut parsed: Vec<OgfVertex> = Vec::with_capacity(vertex_count as usize);

    for vertex in vertices.chunks_exact(vertex_size) {
      parsed.push(OgfVertex::read_from_slice::<T>(vertex, bone_indices_per_vertex));
    }

    Ok(OgfVertices {
      format: vertex_format,
      count: vertex_count,
      vertices: Some(parsed),
    })
  }

  fn read_indices<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<u16>> {
    let indices: Vec<u16> = reader.read_u16_vector::<T>()?;
    reader.assert_read("Expect all data to be read from ogf indices")?;

    Ok(indices)
  }

  fn skin_vertex_layout(vertex_format: u32) -> Option<(usize, usize)> {
    match vertex_format {
      Self::VERTEX_FORMAT_1_LINK | 1 => Some((Self::VERTEX_SIZE_1_LINK, 1)),
      Self::VERTEX_FORMAT_2_LINK | 2 => Some((Self::VERTEX_SIZE_2_LINK, 2)),
      Self::VERTEX_FORMAT_3_LINK | 3 => Some((Self::VERTEX_SIZE_3_LINK, 3)),
      Self::VERTEX_FORMAT_4_LINK | 4 => Some((Self::VERTEX_SIZE_4_LINK, 4)),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use byteorder::WriteBytesExt;
  use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use super::OgfGeometry;

  #[test]
  fn test_read() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "read.chunk");
    let contents: Vec<u8> = geometry_contents()?;
    let mut file = overwrite_generated_test_resource_as_file(&filename)?;

    file.write_all(&contents)?;
    file.flush()?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    assert_eq!(file.bytes_remaining(), contents.len());

    let chunks = ChunkReader::from_slice(file)?.read_children()?;
    let geometry: OgfGeometry =
      OgfGeometry::read_from_chunks::<XRayByteOrder, _>(&chunks)?.expect("geometry chunks are present");

    assert_eq!(geometry.vertex_count, Some(2));
    assert_eq!(geometry.indices, Some(vec![0, 1, 0]));
    assert_eq!(geometry.skin_bone_indices, vec![1, 3, 2, 4]);

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
