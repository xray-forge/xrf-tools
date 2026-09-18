use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
#[serde(rename_all = "camelCase")]
#[display("{r},{g},{b}")]
pub struct RgbColor {
  pub r: f32,
  pub g: f32,
  pub b: f32,
}

impl ChunkReadWrite for RgbColor {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      r: reader.read_f32::<T>()?,
      g: reader.read_f32::<T>()?,
      b: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.r)?;
    writer.write_f32::<T>(self.g)?;
    writer.write_f32::<T>(self.b)?;

    Ok(())
  }
}

impl FromStr for RgbColor {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = string.split(',').map(str::trim).collect();

    if parts.len() != 3 {
      return Err(XrfError::new_parsing_error(
        "Failed to parse rgb color from string, expected 3 numbers",
      ));
    }

    Ok(Self {
      r: parts[0]
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse color R value")))?,
      g: parts[1]
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse color G value")))?,
      b: parts[2]
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse color B value")))?,
    })
  }
}
