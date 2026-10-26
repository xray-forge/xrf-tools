use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

/// A vector as `NET_Packet::w_sdir` writes it: its direction compressed into a `u16` (`pvCompress`), then its length.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaledDirection {
  /// The unit direction as `pvCompress` packs it.
  pub direction: u16,
  pub magnitude: f32,
}

impl ScaledDirection {
  /// The vector `NET_Packet::r_sdir` reads: the decompressed direction scaled by the magnitude.
  pub fn get_vector(&self) -> Vector3d {
    &Vector3d::from_compressed_normal(self.direction) * self.magnitude
  }
}

impl ChunkReadWrite for ScaledDirection {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      direction: reader.read_u16::<T>()?,
      magnitude: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u16::<T>(self.direction)?;
    writer.write_f32::<T>(self.magnitude)?;

    Ok(())
  }
}

impl Display for ScaledDirection {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    write!(formatter, "{},{}", self.direction, self.magnitude)
  }
}

impl FromStr for ScaledDirection {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let (direction, magnitude) = string.split_once(',').ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Failed to parse scaled direction '{string}', expected a packed direction and a magnitude"
      ))
    })?;

    Ok(Self {
      direction: direction
        .trim()
        .parse()
        .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse packed direction: {error}")))?,
      magnitude: magnitude
        .trim()
        .parse()
        .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse direction magnitude: {error}")))?,
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;

  use crate::data::physics::scaled_direction::ScaledDirection;

  #[test]
  fn reads_and_writes_the_six_bytes_of_a_scaled_direction() -> XrfResult {
    let original: ScaledDirection = ScaledDirection {
      direction: 126,
      magnitude: 2.5,
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer.len(), 6);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(ScaledDirection::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  /// 126 in the low bits is the compressed normal straight along y.
  #[test]
  fn reads_the_vector_the_engine_decompresses() {
    let vector: Vector3d = ScaledDirection {
      direction: 126,
      magnitude: 2.5,
    }
    .get_vector();

    assert!(vector.distance(&Vector3d::new(0.0, 2.5, 0.0)) < 1e-5, "{vector}");
  }

  #[test]
  fn parses_what_it_prints() -> XrfResult {
    let original: ScaledDirection = ScaledDirection {
      direction: 8191,
      magnitude: -0.125,
    };

    assert_eq!(original.to_string().parse::<ScaledDirection>()?, original);
    assert!("8191".parse::<ScaledDirection>().is_err());

    Ok(())
  }
}
