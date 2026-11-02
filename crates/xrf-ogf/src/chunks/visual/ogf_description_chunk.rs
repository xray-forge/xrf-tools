use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::chunks::visual::ogf_description_time_width::OgfDescriptionTimeWidth;

/// Who built a visual and with what, `ogf_desc` (`xray-16/src/xrCore/FMesh.cpp`): three names, each followed by a time,
/// after the source file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfDescriptionChunk {
  pub source_file: String,
  pub convertor: String,
  /// Unix seconds, as every time here.
  pub built_at: u64,
  pub creator: String,
  pub created_at: u64,
  pub editor: String,
  pub edited_at: u64,
  /// How wide the file stores the times, which a write keeps.
  pub time_width: OgfDescriptionTimeWidth,
}

impl OgfDescriptionChunk {
  pub const CHUNK_ID: u32 = 18;

  fn read_with<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    time_width: OgfDescriptionTimeWidth,
  ) -> XrfResult<Self> {
    let time = |reader: &mut ChunkReader<D>| -> XrfResult<u64> {
      Ok(match time_width {
        OgfDescriptionTimeWidth::Narrow => u64::from(reader.read_u32::<T>()?),
        OgfDescriptionTimeWidth::Wide => reader.read_u64::<T>()?,
      })
    };

    Ok(Self {
      source_file: reader.read_w1251_string()?,
      convertor: reader.read_w1251_string()?,
      built_at: time(reader)?,
      creator: reader.read_w1251_string()?,
      created_at: time(reader)?,
      editor: reader.read_w1251_string()?,
      edited_at: time(reader)?,
      time_width,
    })
  }
}

impl ChunkReadWrite for OgfDescriptionChunk {
  /// Reads the description in whichever time width the chunk ends on exactly, narrow first.
  ///
  /// # Errors
  ///
  /// Returns an error when neither width reads the chunk whole, which no exporter's description does.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    reader.reset_pos()?;

    let size: u64 = reader.read_bytes_remain();

    for time_width in [OgfDescriptionTimeWidth::Narrow, OgfDescriptionTimeWidth::Wide] {
      reader.reset_pos()?;

      if let Ok(description) = Self::read_with::<T, D>(reader, time_width)
        && reader.is_ended()
      {
        return Ok(description);
      }
    }

    Err(XrfError::new_invalid_error(format!(
      "Unexpected ogf description of {size} bytes, which neither 32-bit nor 64-bit times read whole"
    )))
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let time = |writer: &mut ChunkWriter, value: u64| -> XrfResult {
      match self.time_width {
        OgfDescriptionTimeWidth::Narrow => writer.write_u32::<T>(u32::try_from(value).map_err(|_| {
          XrfError::new_invalid_error(format!("Time {value} does not fit the description's 32-bit times"))
        })?)?,
        OgfDescriptionTimeWidth::Wide => writer.write_u64::<T>(value)?,
      }

      Ok(())
    };

    writer.write_w1251_string(&self.source_file)?;
    writer.write_w1251_string(&self.convertor)?;
    time(writer, self.built_at)?;
    writer.write_w1251_string(&self.creator)?;
    time(writer, self.created_at)?;
    writer.write_w1251_string(&self.editor)?;
    time(writer, self.edited_at)?;

    Ok(())
  }
}
