use std::fmt::{Display, Formatter};
use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::vector_from_string_sized;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct U32Bytes(pub u8, pub u8, pub u8, pub u8);

impl ChunkReadWrite for U32Bytes {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(U32Bytes(
      reader.read_u8()?,
      reader.read_u8()?,
      reader.read_u8()?,
      reader.read_u8()?,
    ))
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(self.0)?;
    writer.write_u8(self.1)?;
    writer.write_u8(self.2)?;
    writer.write_u8(self.3)?;

    Ok(())
  }
}

impl Display for U32Bytes {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{},{},{},{}", self.0, self.1, self.2, self.3,)
  }
}

impl FromStr for U32Bytes {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let values: Vec<u8> = vector_from_string_sized(string, 4)?;

    Ok(Self(values[0], values[1], values[2], values[3]))
  }
}

impl From<(u8, u8, u8, u8)> for U32Bytes {
  fn from(tuple: (u8, u8, u8, u8)) -> Self {
    Self(tuple.0, tuple.1, tuple.2, tuple.3)
  }
}
