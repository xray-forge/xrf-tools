use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

/// Link to a sliding window table stored outside the visual, `OGF_SWICONTAINER`: an index into the level's `fsL_SWIS`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfSwiContainerChunk {
  pub ext_swib_index: u32,
}

impl OgfSwiContainerChunk {
  /// `OGF_SWICONTAINER` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 20;
}

impl ChunkReadWrite for OgfSwiContainerChunk {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let container: Self = Self {
      ext_swib_index: reader.read_u32()?,
    };

    reader.assert_read("Expect all data to be read from ogf swi container")?;

    Ok(container)
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.ext_swib_index)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter};
  use xrf_error::XrfResult;

  use super::OgfSwiContainerChunk;

  #[test]
  fn test_read_write() -> XrfResult {
    let original: OgfSwiContainerChunk = OgfSwiContainerChunk { ext_swib_index: 147 };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write(&mut writer)?;

    let contents: Vec<u8> = writer.flush_chunk_into_buffer(OgfSwiContainerChunk::CHUNK_ID)?;
    let mut reader: ChunkReader<_> = ChunkReader::from_vec(contents)?
      .read_children()?
      .into_iter()
      .next()
      .expect("the chunk written");

    assert_eq!(OgfSwiContainerChunk::read(&mut reader)?, original);

    Ok(())
  }
}
