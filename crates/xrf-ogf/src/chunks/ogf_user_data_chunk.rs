use std::io::Write;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::{decode_bytes_to_string, encode_string_to_bytes, new_windows1251_encoder};

/// Free-form ini text attached to a skeleton, `OGF_S_USERDATA` in the engine.
#[derive(Debug)]
pub struct OgfUserDataChunk {
  pub user_data: String,
}

impl OgfUserDataChunk {
  pub const CHUNK_ID: u32 = 17;
}

impl ChunkReadWrite for OgfUserDataChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let bytes: Vec<u8> = reader.read_remaining()?;

    Ok(Self {
      user_data: decode_bytes_to_string(&bytes, new_windows1251_encoder())?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&encode_string_to_bytes(&self.user_data, new_windows1251_encoder())?)?;

    Ok(())
  }
}
