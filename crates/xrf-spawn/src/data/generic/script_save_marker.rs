use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_format_size;

/// The check a script server class closes its own fields with: a `u16` counting the bytes written since the marker
/// opened (`set_save_marker` in vanilla scripts, `closeSaveMarker` in xrf-engine). Both loaders assert it, so it is
/// derived from the fields it closes rather than kept beside them, and an edited field writes a marker that still holds.
pub(crate) struct ScriptSaveMarker;

impl ScriptSaveMarker {
  /// Writes the fields `write` puts down, closed by their byte count.
  ///
  /// # Errors
  ///
  /// Returns an error when `write` fails, or when the fields outgrow the marker's `u16`.
  pub fn write<T: ByteOrder>(
    writer: &mut ChunkWriter,
    owner: &str,
    write: impl FnOnce(&mut ChunkWriter) -> XrfResult,
  ) -> XrfResult {
    let opened: usize = writer.bytes_written();

    write(writer)?;
    writer.write_u16::<T>(to_format_size(writer.bytes_written() - opened, owner)?)?;

    Ok(())
  }

  /// Reads the fields `read` takes, then the marker closing them, refusing a marker that counts another size.
  ///
  /// # Errors
  ///
  /// Returns an error when `read` fails, the marker is missing, or it does not count the bytes `read` took.
  pub fn read<T: ByteOrder, D: ChunkDataSource, R>(
    reader: &mut ChunkReader<D>,
    owner: &str,
    read: impl FnOnce(&mut ChunkReader<D>) -> XrfResult<R>,
  ) -> XrfResult<R> {
    let opened: u64 = reader.read_bytes_len();
    let fields: R = read(reader)?;
    let counted: u64 = reader.read_bytes_len() - opened;
    let marker: u16 = reader.read_u16::<T>()?;

    if u64::from(marker) != counted {
      return Err(XrfError::new_invalid_error(format!(
        "{owner} save marker counts {marker} bytes, its fields hold {counted}"
      )));
    }

    Ok(fields)
  }
}

#[cfg(test)]
mod tests {
  use byteorder::{ReadBytesExt, WriteBytesExt};
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::data::generic::script_save_marker::ScriptSaveMarker;

  #[test]
  fn counts_the_fields_it_closes() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_u8(7)?;
    ScriptSaveMarker::write::<XRayByteOrder>(&mut writer, "test", |writer| {
      writer.write_u8(1)?;
      writer.write_w1251_string("hint")?;

      Ok(())
    })?;

    // The byte before the marker opened is not counted: one flag and a four-letter string with its terminator.
    assert_eq!(writer.buffer[writer.buffer.len() - 2..], [6, 0]);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(reader.read_u8()?, 7);
    assert_eq!(
      ScriptSaveMarker::read::<XRayByteOrder, _, _>(&mut reader, "test", |reader| {
        Ok((reader.read_u8()?, reader.read_w1251_string()?))
      })?,
      (1, String::from("hint"))
    );

    Ok(())
  }

  #[test]
  fn refuses_a_marker_counting_another_size() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[1, 26, 0])?;
    let error: String =
      ScriptSaveMarker::read::<XRayByteOrder, _, _>(&mut reader, "test", |reader| Ok(reader.read_u8()?))
        .expect_err("a marker that counts more than its fields")
        .to_string();

    assert!(
      error.contains("save marker counts 26 bytes, its fields hold 1"),
      "{error}"
    );

    Ok(())
  }
}
