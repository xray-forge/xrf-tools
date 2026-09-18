use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// `hdrLEVEL` in c++ codebase, stored in the `fsL_HEADER` chunk of the `level` file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelHeaderChunk {
  pub xrlc_version: u16,
  pub xrlc_quality: u16,
}

impl LevelHeaderChunk {
  pub const CHUNK_ID: u32 = 1;
}

impl ChunkReadWrite for LevelHeaderChunk {
  /// Read level header data from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      xrlc_version: reader.read_u16::<T>()?,
      xrlc_quality: reader.read_u16::<T>()?,
    })
  }

  /// Write level header data into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.xrlc_version)?;
    writer.write_u16::<T>(self.xrlc_quality)?;

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

  use crate::level::level_header_chunk::LevelHeaderChunk;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: LevelHeaderChunk = LevelHeaderChunk {
      xrlc_version: 14,
      xrlc_quality: 1,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 4);

    writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      LevelHeaderChunk::CHUNK_ID,
    )?;

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(LevelHeaderChunk::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }
}
