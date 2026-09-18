use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// The children a visual links to rather than carries, `OGF_CHILDREN_L`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfChildrenLinkChunk {
  pub children: Vec<u32>,
}

impl OgfChildrenLinkChunk {
  /// `OGF_CHILDREN_L` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 10;

  /// Bytes one child id occupies.
  pub const CHILD_SIZE: u64 = 4;
}

impl ChunkReadWrite for OgfChildrenLinkChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let count: u32 = reader.read_u32::<T>()?;
    let mut children: Vec<u32> = reader.new_bounded_vec(count as u64, Self::CHILD_SIZE, "ogf linked children")?;

    for _ in 0..count {
      children.push(reader.read_u32::<T>()?);
    }

    reader.assert_read("Expect all data to be read from ogf linked children chunk")?;

    Ok(Self { children })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.children.len() as u32)?;

    for child in &self.children {
      writer.write_u32::<T>(*child)?;
    }

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

  use crate::chunks::ogf_children_link_chunk::OgfChildrenLinkChunk;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: OgfChildrenLinkChunk = OgfChildrenLinkChunk {
      children: vec![0, 7, 4294967295],
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 4 + 3 * 4);

    writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      OgfChildrenLinkChunk::CHUNK_ID,
    )?;

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(OgfChildrenLinkChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_reads_a_visual_that_links_nothing() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    OgfChildrenLinkChunk { children: Vec::new() }.write::<XRayByteOrder>(&mut writer)?;

    let filename: String = String::from("empty.chunk");

    writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      OgfChildrenLinkChunk::CHUNK_ID,
    )?;

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert!(
      OgfChildrenLinkChunk::read::<XRayByteOrder, _>(&mut reader)?
        .children
        .is_empty()
    );

    Ok(())
  }
}
