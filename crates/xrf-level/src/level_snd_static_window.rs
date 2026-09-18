use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// A window a static sound answers to, as a pair the engine compares a clock against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SndStaticWindow {
  pub from: u32,
  pub to: u32,
}

impl SndStaticWindow {
  /// Bytes a window occupies.
  pub const SERIALIZED_SIZE: u64 = 4 + 4;

  /// Whether the window bounds anything, which is either end being non-zero.
  pub const fn is_bounded(&self) -> bool {
    self.from != 0 || self.to != 0
  }
}

impl ChunkReadWrite for SndStaticWindow {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      from: reader.read_u32::<T>()?,
      to: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.from)?;
    writer.write_u32::<T>(self.to)?;

    Ok(())
  }
}
