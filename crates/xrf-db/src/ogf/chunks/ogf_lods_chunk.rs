use std::io::Write;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::{decode_bytes_to_string, encode_string_to_bytes, new_windows1251_encoder};

/// Level of detail visual of a skeleton, `OGF_S_LODS` in the engine.
///
/// **Not a null terminated string.** `SkeletonCustom.cpp` reads it with `r_string`, which takes a line
/// rather than stopping at a null, and files in practice contain the text with no terminator at all. So
/// the whole payload is taken as text, which is also how the engine treats it.
#[derive(Debug)]
pub struct OgfLodsChunk {
  pub lods: String,
}

impl OgfLodsChunk {
  pub const CHUNK_ID: u32 = 23;
}

impl ChunkReadWrite for OgfLodsChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let bytes: Vec<u8> = reader.read_remaining()?;

    Ok(Self {
      lods: decode_bytes_to_string(&bytes, new_windows1251_encoder())?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&encode_string_to_bytes(&self.lods, new_windows1251_encoder())?)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use super::OgfLodsChunk;

  /// Round trip the given text through a real chunk, the way the reader sees it on disk.
  fn write_then_read(name: &str, lods: &str) -> XrfResult<OgfLodsChunk> {
    let filename: String = build_relative_test_sample_file_path(file!(), name);
    let mut writer: ChunkWriter = ChunkWriter::new();

    OgfLodsChunk {
      lods: String::from(lods),
    }
    .write::<XRayByteOrder>(&mut writer)?;

    let contents: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(OgfLodsChunk::CHUNK_ID)?;
    let mut file = overwrite_generated_test_resource_as_file(&filename)?;

    file.write_all(&contents)?;
    file.flush()?;

    let slice: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut chunk: ChunkReader = ChunkReader::from_slice(slice)?
      .read_children()?
      .into_iter()
      .next()
      .expect("expect the written chunk to be present");

    OgfLodsChunk::read::<XRayByteOrder, _>(&mut chunk)
  }

  #[test]
  fn round_trips_text_without_a_null_terminator() -> XrfResult {
    // Real files store the visual path with no terminator, which a stringZ read cannot handle at all.
    let path: &str = r"dynamics\weapons\wpn_abakan\wpn_abakan_lod";

    assert_eq!(write_then_read("unterminated.chunk", path)?.lods, path);

    Ok(())
  }

  #[test]
  fn round_trips_an_empty_payload() -> XrfResult {
    assert_eq!(write_then_read("empty.chunk", "")?.lods, "");

    Ok(())
  }
}
