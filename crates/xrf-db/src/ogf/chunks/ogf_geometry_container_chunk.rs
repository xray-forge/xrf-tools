use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Where a visual's geometry sits in buffers the visual does not own, `OGF_GCONTAINER`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OgfGeometryContainerChunk {
  pub vertex_buffer_id: u32,
  pub vertex_base: u32,
  pub vertex_count: u32,
  pub index_buffer_id: u32,
  pub index_base: u32,
  pub index_count: u32,
}

impl OgfGeometryContainerChunk {
  /// `OGF_GCONTAINER` (`xray-16/src/xrCore/FMesh.hpp`). Not adjacent to `OGF_VCONTAINER` (7) and
  /// `OGF_ICONTAINER` (8), which the engine marks unused.
  pub const CHUNK_ID: u32 = 21;

  /// `OGF_FASTPATH`, which is a container chunk holding another [`Self::CHUNK_ID`] rather than the record itself
  /// (`xray-16/src/Layers/xrRender/FVisual.cpp`).
  pub const FASTPATH_CHUNK_ID: u32 = 22;

  /// Bytes the record occupies in the file.
  pub const SERIALIZED_SIZE: u64 = 6 * 4;

  /// Triangles drawn from the index range, the engine's `dwPrimitives`.
  pub const fn get_primitive_count(&self) -> u32 {
    self.index_count / 3
  }
}

impl ChunkReadWrite for OgfGeometryContainerChunk {
  /// Read a geometry container from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let container: Self = Self {
      vertex_buffer_id: reader.read_u32::<T>()?,
      vertex_base: reader.read_u32::<T>()?,
      vertex_count: reader.read_u32::<T>()?,
      index_buffer_id: reader.read_u32::<T>()?,
      index_base: reader.read_u32::<T>()?,
      index_count: reader.read_u32::<T>()?,
    };

    reader.assert_read("Expect all data to be read from ogf geometry container")?;

    Ok(container)
  }

  /// Write a geometry container into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.vertex_buffer_id)?;
    writer.write_u32::<T>(self.vertex_base)?;
    writer.write_u32::<T>(self.vertex_count)?;
    writer.write_u32::<T>(self.index_buffer_id)?;
    writer.write_u32::<T>(self.index_base)?;
    writer.write_u32::<T>(self.index_count)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::ogf::chunks::ogf_geometry_container_chunk::OgfGeometryContainerChunk;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: OgfGeometryContainerChunk = OgfGeometryContainerChunk {
      vertex_buffer_id: 3,
      vertex_base: 1024,
      vertex_count: 512,
      index_buffer_id: 1,
      index_base: 2048,
      index_count: 900,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(
      writer.bytes_written() as u64,
      OgfGeometryContainerChunk::SERIALIZED_SIZE
    );

    writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      OgfGeometryContainerChunk::CHUNK_ID,
    )?;

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(
      OgfGeometryContainerChunk::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }

  #[test]
  fn test_primitive_count_is_triangles() {
    let container: OgfGeometryContainerChunk = OgfGeometryContainerChunk {
      vertex_buffer_id: 0,
      vertex_base: 0,
      vertex_count: 3,
      index_buffer_id: 0,
      index_base: 0,
      index_count: 900,
    };

    assert_eq!(container.get_primitive_count(), 300);
  }
}
