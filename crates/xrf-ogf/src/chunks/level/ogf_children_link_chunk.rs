use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::{XrfError, XrfResult};

/// The children a visual links to rather than carries, `OGF_CHILDREN_L`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfChildrenLinkChunk {
  pub children: Vec<u32>,
}

impl OgfChildrenLinkChunk {
  /// `OGF_CHILDREN_L` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 10;
}

impl ChunkReadWrite for OgfChildrenLinkChunk {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let count: u32 = reader.read_u32()?;
    let children: Vec<u32> = reader.read_records(count.into(), "ogf linked children")?;

    reader.assert_read("Expect all data to be read from ogf linked children chunk")?;

    Ok(Self { children })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    let count: u32 = u32::try_from(self.children.len()).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "A visual links {} children, more than a u32 counts",
        self.children.len()
      ))
    })?;

    writer.write_u32(count)?;
    writer.write_records(&self.children)
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter};
  use xrf_error::XrfResult;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::chunks::level::ogf_children_link_chunk::OgfChildrenLinkChunk;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: OgfChildrenLinkChunk = OgfChildrenLinkChunk {
      children: vec![0, 7, 4294967295],
    };

    original.write(&mut writer)?;

    assert_eq!(writer.bytes_written(), 4 + 3 * 4);

    writer.flush_chunk_into(
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      OgfChildrenLinkChunk::CHUNK_ID,
    )?;

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(OgfChildrenLinkChunk::read(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_reads_a_visual_that_links_nothing() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    OgfChildrenLinkChunk { children: Vec::new() }.write(&mut writer)?;

    let filename: String = String::from("empty.chunk");

    writer.flush_chunk_into(
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      OgfChildrenLinkChunk::CHUNK_ID,
    )?;

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert!(OgfChildrenLinkChunk::read(&mut reader)?.children.is_empty());

    Ok(())
  }
}
