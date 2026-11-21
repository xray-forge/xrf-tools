use xrf_chunk::{
  ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayRead, XRayWrite,
};
use xrf_error::{XrfError, XrfResult};

use crate::object::editable_time_width::EditableTimeWidth;

/// Who made an editable object and when, `EOBJ_CHUNK_DESC`. The times are Unix seconds, stored at the width of the
/// `time_t` the saving editor was built with, 32 or 64 bits; the SDK's own objects hold both.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableDescription {
  pub creator: String,
  pub created_at: u64,
  pub modifier: String,
  pub modified_at: u64,
  pub time_width: EditableTimeWidth,
}

impl EditableDescription {
  /// Reads one time stamp at `width`.
  fn read_time<D: ChunkDataSource>(reader: &mut ChunkReader<D>, width: EditableTimeWidth) -> XrfResult<u64> {
    Ok(match width {
      EditableTimeWidth::Seconds32 => u64::from(reader.read_u32()?),
      EditableTimeWidth::Seconds64 => reader.read_u64()?,
    })
  }

  /// Writes one time stamp at this description's width, refusing one too large for it.
  fn write_time(&self, writer: &mut ChunkWriter, value: u64) -> XrfResult {
    match self.time_width {
      EditableTimeWidth::Seconds32 => writer.write_u32(u32::try_from(value).map_err(|_| {
        XrfError::new_invalid_error(format!(
          "Time stamp {value} does not fit the description's 32-bit width"
        ))
      })?)?,
      EditableTimeWidth::Seconds64 => writer.write_u64(value)?,
    }

    Ok(())
  }
}

impl ChunkReadWrite for EditableDescription {
  /// Reads the creator, then takes the width from what is left, which only one width accounts for.
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let creator: String = reader.read_w1251_string()?;
    let tail: Vec<u8> = reader.read_remaining()?;
    let time_width: EditableTimeWidth = EditableTimeWidth::detect(&tail)?;
    let mut tail: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(tail)?;

    Ok(Self {
      creator,
      created_at: Self::read_time(&mut tail, time_width)?,
      modifier: tail.read_w1251_string()?,
      modified_at: Self::read_time(&mut tail, time_width)?,
      time_width,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.creator)?;
    self.write_time(writer, self.created_at)?;
    writer.write_w1251_string(&self.modifier)?;
    self.write_time(writer, self.modified_at)?;

    Ok(())
  }
}
