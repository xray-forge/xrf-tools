use byteorder::ByteOrder;
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::narrow_format_size;

/// The check a script server class closes its own fields with: a `u16` counting the bytes written since the marker
/// opened (`set_save_marker` in vanilla scripts, `closeSaveMarker` in xrf-engine). Both loaders assert it, so it is
/// derived from the fields it closes rather than kept beside them, and an edited field writes a marker that still holds.
///
/// The count is the packet's own (`w_tell`), so in fields it counts the bytes the values stand for, as the engine's
/// does with an ini stream attached.
pub(crate) struct ScriptSaveMarker;

impl ScriptSaveMarker {
  /// Writes the fields `write` puts down, closed by their byte count.
  ///
  /// # Errors
  ///
  /// Returns an error when `write` fails, or when the fields outgrow the marker's `u16`.
  pub(crate) fn write<T: ByteOrder, W: ChunkFieldWriter>(
    writer: &mut W,
    owner: &str,
    write: impl FnOnce(&mut W) -> XrfResult,
  ) -> XrfResult {
    let opened: usize = writer.get_position();

    write(writer)?;
    writer.write_u16::<T>(narrow_format_size(writer.get_position() - opened, owner)?)
  }

  /// Reads the fields `read` takes, then the marker closing them, refusing a marker that counts another size.
  ///
  /// # Errors
  ///
  /// Returns an error when `read` fails, the marker is missing, or it does not count the bytes `read` took.
  pub(crate) fn read<T: ByteOrder, R: ChunkFieldReader, V>(
    reader: &mut R,
    owner: &str,
    read: impl FnOnce(&mut R) -> XrfResult<V>,
  ) -> XrfResult<V> {
    let opened: usize = reader.get_position();
    let fields: V = read(reader)?;
    let counted: usize = reader.get_position() - opened;
    let marker: u16 = reader.read_u16::<T>()?;

    if usize::from(marker) != counted {
      return Err(XrfError::new_invalid_error(format!(
        "{owner} save marker counts {marker} bytes, its fields hold {counted}"
      )));
    }

    Ok(fields)
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{
    ChunkFieldReader, ChunkFieldWriter, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
  };
  use xrf_error::XrfResult;

  use crate::data::generic::script_save_marker::ScriptSaveMarker;

  #[test]
  fn counts_the_fields_it_closes() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    ChunkFieldWriter::write_u8(&mut writer, 7)?;
    ScriptSaveMarker::write::<XRayByteOrder, _>(&mut writer, "test", |writer| {
      ChunkFieldWriter::write_u8(writer, 1)?;
      ChunkFieldWriter::write_w1251_string(writer, "hint")
    })?;

    // The byte before the marker opened is not counted: one flag and a four-letter string with its terminator.
    assert_eq!(writer.buffer[writer.buffer.len() - 2..], [6, 0]);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(ChunkFieldReader::read_u8(&mut reader)?, 7);
    assert_eq!(
      ScriptSaveMarker::read::<XRayByteOrder, _, _>(&mut reader, "test", |reader| {
        Ok((
          ChunkFieldReader::read_u8(reader)?,
          ChunkFieldReader::read_w1251_string(reader)?,
        ))
      })?,
      (1, String::from("hint"))
    );

    Ok(())
  }

  #[test]
  fn refuses_a_marker_counting_another_size() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[1, 26, 0])?;
    let error: String = ScriptSaveMarker::read::<XRayByteOrder, _, _>(&mut reader, "test", ChunkFieldReader::read_u8)
      .expect_err("a marker that counts more than its fields")
      .to_string();

    assert!(
      error.contains("save marker counts 26 bytes, its fields hold 1"),
      "{error}"
    );

    Ok(())
  }
}
