use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteOptional, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::constants::NIL;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Display)]
#[serde(rename_all = "camelCase")]
#[display("{year},{month},{day},{hour},{minute},{second},{millis}")]
pub struct Time {
  pub year: u8,
  pub month: u8,
  pub day: u8,
  pub hour: u8,
  pub minute: u8,
  pub second: u8,
  pub millis: u16,
}

impl ChunkReadWriteOptional for Time {
  /// Read optional time object from the chunk.
  fn read_optional<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Option<Self>> {
    if reader.read_u8()? == 1 {
      Ok(Some(Self::read::<T, _>(reader)?))
    } else {
      Ok(None)
    }
  }

  /// Write optional time object into the writer.
  fn write_optional<T: ByteOrder>(writer: &mut ChunkWriter, time: Option<&Self>) -> XrfResult {
    if let Some(time) = time {
      writer.write_u8(1)?;

      time.write::<T>(writer)?;
    } else {
      writer.write_u8(0)?;
    }

    Ok(())
  }
}

impl ChunkReadWrite for Time {
  /// Read time object from chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let year: u8 = reader.read_u8()?;
    let month: u8 = reader.read_u8()?;
    let day: u8 = reader.read_u8()?;
    let hour: u8 = reader.read_u8()?;
    let minute: u8 = reader.read_u8()?;
    let second: u8 = reader.read_u8()?;
    let millis: u16 = reader.read_u16::<T>()?;

    Ok(Self {
      year,
      month,
      day,
      hour,
      minute,
      second,
      millis,
    })
  }

  /// Write time object into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u8(self.year)?;
    writer.write_u8(self.month)?;
    writer.write_u8(self.day)?;
    writer.write_u8(self.hour)?;
    writer.write_u8(self.minute)?;
    writer.write_u8(self.second)?;
    writer.write_u16::<T>(self.millis)?;

    Ok(())
  }
}

impl Time {
  /// Cast optional time object to serialized string.
  pub fn export_to_string(time: Option<&Self>) -> String {
    time.as_ref().map_or(String::from(NIL), |value| value.to_string())
  }

  /// Import optional time from string value.
  pub fn from_str_optional(value: &str) -> XrfResult<Option<Self>> {
    if value.trim() == NIL {
      return Ok(None);
    }

    Self::from_str(value).map(Some)
  }
}

impl FromStr for Time {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = string.split(',').map(str::trim).collect();

    if parts.len() != 7 {
      return Err(XrfError::new_parsing_error("Failed to parse time object from string"));
    }

    Ok(Self {
      year: parts[0]
        .parse()
        .or(Err(XrfError::new_parsing_error("Failed to parse years value")))?,
      month: parts[1]
        .parse()
        .or(Err(XrfError::new_parsing_error("Failed to parse months value")))?,
      day: parts[2]
        .parse()
        .or(Err(XrfError::new_parsing_error("Failed to parse days value")))?,
      hour: parts[3]
        .parse()
        .or(Err(XrfError::new_parsing_error("Failed to parse hours value")))?,
      minute: parts[4]
        .parse()
        .or(Err(XrfError::new_parsing_error("Failed to parse minutes value")))?,
      second: parts[5]
        .parse()
        .or(Err(XrfError::new_parsing_error("Failed to parse seconds value")))?,
      millis: parts[6]
        .parse()
        .or(Err(XrfError::new_parsing_error("Failed to parse millis value")))?,
    })
  }
}

#[cfg(test)]
impl Time {
  pub fn new_mock() -> Self {
    Self {
      year: 4,
      month: 6,
      day: 14,
      hour: 12,
      minute: 30,
      second: 10,
      millis: 510,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::str::FromStr;

  use serde_json::to_string_pretty;
  use xrf_chunk::{
    ChunkReadWrite, ChunkReadWriteOptional, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
  };
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::generic::time::Time;

  /// Reads a chunked value from bytes rather than from a file.
  ///
  /// This is what an archived asset requires: a `.db` volume entry has no file to slice, so its decompressed bytes are the
  /// only way in. Until `ChunkReadWrite` became generic over `ChunkDataSource` this could not compile at all, which is why
  /// nothing had ever read a chunked format out of an archive.
  #[test]
  fn reads_from_bytes_as_well_as_from_a_file() -> XrfResult {
    let original: Time = Time {
      year: 22,
      month: 10,
      day: 24,
      hour: 20,
      minute: 30,
      second: 50,
      millis: 250,
    };

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = Vec::new();

    writer.flush_chunk_into::<XRayByteOrder>(&mut bytes, 0)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?.read_child_by_index(0)?;

    assert_eq!(Time::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: Time = Time {
      year: 22,
      month: 10,
      day: 24,
      hour: 20,
      minute: 30,
      second: 50,
      millis: 250,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 8);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 8);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 8 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Time::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_optional_some() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_optional_some.chunk");

    let original: Time = Time {
      year: 22,
      month: 10,
      day: 24,
      hour: 20,
      minute: 30,
      second: 50,
      millis: 250,
    };

    writer.write_xr_optional::<XRayByteOrder, _>(Some(&original))?;

    assert_eq!(writer.bytes_written(), 9);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 9);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 9 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Time::read_optional::<XRayByteOrder, _>(&mut reader)?, Some(original));

    Ok(())
  }

  #[test]
  fn test_read_write_optional_none() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_optional_none.chunk");

    Time::write_optional::<XRayByteOrder>(&mut writer, None)?;

    assert_eq!(writer.bytes_written(), 1);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 1);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 1 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(Time::read_optional::<XRayByteOrder, _>(&mut reader)?, None);

    Ok(())
  }

  #[test]
  fn test_import_export_to_str() -> XrfResult {
    let original: Time = Time {
      year: 20,
      month: 6,
      day: 1,
      hour: 15,
      minute: 15,
      second: 23,
      millis: 100,
    };

    assert_eq!(Time::export_to_string(Some(&original)), "20,6,1,15,15,23,100");
    assert_eq!(Time::from_str_optional("20,6,1,15,15,23,100")?, Some(original));
    assert_eq!(Time::export_to_string(None), "nil");
    assert_eq!(Time::from_str_optional("nil")?, None);

    Ok(())
  }

  #[test]
  fn test_from_to_str() -> XrfResult {
    let original: Time = Time {
      year: 22,
      month: 6,
      day: 1,
      hour: 15,
      minute: 15,
      second: 23,
      millis: 100,
    };

    assert_eq!(original.to_string(), "22,6,1,15,15,23,100");
    assert_eq!(Time::from_str("22,6,1,15,15,23,100").unwrap(), original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: Time = Time {
      year: 22,
      month: 6,
      day: 1,
      hour: 15,
      minute: 16,
      second: 45,
      millis: 100,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<Time>(&serialized)?);

    Ok(())
  }
}
