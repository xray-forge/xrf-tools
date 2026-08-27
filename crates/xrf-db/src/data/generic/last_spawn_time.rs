use byteorder::{ByteOrder, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::constants::NIL;
use crate::data::generic::time::Time;

/// Trailing spawn time a fork's script class appends after the engine server class payload.
///
/// The engine registers one server class per zone family; `se_zones.script` layers a script class
/// over it and may or may not extend `STATE_Write`. A spawn file is a compiled artifact, so whether
/// the tail exists is a property of the stored bytes, not of the section name or the installation
/// config: CoC and Anomaly ship the same `generator_dust_static` object with and without it.
/// The object chunk's remaining byte budget is the only authority.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum LastSpawnTime {
  /// The script class wrote nothing past the engine payload.
  Absent,
  /// The script class wrote its flag with no time behind it.
  Unset,
  /// The script class wrote its flag and a time.
  Set(Time),
}

impl LastSpawnTime {
  /// Serialized form of an absent tail, distinct from `NIL`, which is a written flag of zero.
  pub const ABSENT: &'static str = "absent";

  /// Cast an optional ltx value to a tail, where a missing key means the script wrote nothing.
  pub fn from_str_optional(value: Option<&str>) -> XrfResult<Self> {
    Ok(match value.map(str::trim) {
      None | Some(Self::ABSENT) => Self::Absent,
      Some(NIL) => Self::Unset,
      Some(value) => Self::Set(value.parse()?),
    })
  }

  /// Cast the tail to its ltx value, where an absent tail still round trips through a written key.
  pub fn to_ltx_string(&self) -> String {
    match self {
      Self::Absent => String::from(Self::ABSENT),
      Self::Unset => String::from(NIL),
      Self::Set(time) => time.to_string(),
    }
  }
}

impl ChunkReadWrite for LastSpawnTime {
  /// Read the trailing spawn time from whatever budget the object chunk has left.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    if reader.read_bytes_remain() == 0 {
      return Ok(Self::Absent);
    }

    Ok(match reader.read_xr_optional::<T, Time>()? {
      Some(time) => Self::Set(time),
      None => Self::Unset,
    })
  }

  /// Write the trailing spawn time back exactly as it was stored.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    match self {
      Self::Absent => {}
      Self::Unset => writer.write_u8(0)?,
      Self::Set(time) => {
        writer.write_u8(1)?;
        time.write::<T>(writer)?;
      }
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

  use crate::data::generic::last_spawn_time::LastSpawnTime;
  use crate::data::generic::time::Time;

  fn read_back(original: &LastSpawnTime, name: &str, expected_size: usize) -> XrfResult<LastSpawnTime> {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), name);

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), expected_size);

    writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    LastSpawnTime::read::<XRayByteOrder, _>(&mut reader)
  }

  #[test]
  fn test_read_write_absent() -> XrfResult {
    let original: LastSpawnTime = LastSpawnTime::Absent;

    assert_eq!(read_back(&original, "absent.chunk", 0)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_unset() -> XrfResult {
    let original: LastSpawnTime = LastSpawnTime::Unset;

    assert_eq!(read_back(&original, "unset.chunk", 1)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_set() -> XrfResult {
    let original: LastSpawnTime = LastSpawnTime::Set(Time::new_mock());

    assert_eq!(read_back(&original, "set.chunk", 9)?, original);

    Ok(())
  }

  #[test]
  fn test_ltx_round_trip() -> XrfResult {
    for original in [
      LastSpawnTime::Absent,
      LastSpawnTime::Unset,
      LastSpawnTime::Set(Time::new_mock()),
    ] {
      let serialized: String = original.to_ltx_string();

      assert_eq!(LastSpawnTime::from_str_optional(Some(&serialized))?, original);
    }

    Ok(())
  }

  #[test]
  fn test_missing_ltx_key_is_absent() -> XrfResult {
    assert_eq!(LastSpawnTime::from_str_optional(None)?, LastSpawnTime::Absent);

    Ok(())
  }
}
