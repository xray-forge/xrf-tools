use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::narrow_format_size;

/// The check a script server class closes its own fields with: a `u16` counting the bytes since the marker opened
/// (`set_save_marker` in vanilla scripts, `closeSaveMarker` in xrf-engine). Both loaders assert it, so it is derived
/// from the fields rather than stored, and an edited field still writes a valid marker.
///
/// The count is the packet's own (`w_tell`): in field form it counts the bytes the values stand for, as the engine
/// does with an ini stream attached.
pub(crate) struct ScriptSaveMarker;

impl ScriptSaveMarker {
  /// Writes the fields `write` puts down, closed by their byte count.
  ///
  /// # Errors
  ///
  /// Returns an error when `write` fails, or when the fields outgrow the marker's `u16`.
  pub(crate) fn write<W: ChunkFieldWriter>(
    writer: &mut W,
    owner: &str,
    write: impl FnOnce(&mut W) -> XrfResult,
  ) -> XrfResult {
    let opened: usize = writer.get_position();

    write(writer)?;
    writer.write_u16(narrow_format_size(writer.get_position() - opened, owner)?)
  }

  /// Reads the fields `read` takes, then the marker closing them, refusing a marker that counts another size.
  ///
  /// # Errors
  ///
  /// Returns an error when `read` fails, the marker is missing, or it does not count the bytes `read` took.
  pub(crate) fn read<R: ChunkFieldReader, V>(
    reader: &mut R,
    owner: &str,
    read: impl FnOnce(&mut R) -> XrfResult<V>,
  ) -> XrfResult<V> {
    let opened: usize = reader.get_position();
    let fields: V = read(reader)?;
    let counted: usize = reader.get_position() - opened;
    let marker: u16 = reader.read_u16()?;

    if usize::from(marker) != counted {
      return Err(XrfError::new_invalid_error(format!(
        "{owner} save marker counts {marker} bytes, its fields hold {counted}"
      )));
    }

    Ok(fields)
  }

  /// Reads the fields as [`Self::read`] does, or takes the constructor's values in the editor's field form when the
  /// fields end where they would begin. Some SDK scripts return from `STATE_Read` before their own fields under
  /// `editor()` (`se_actor`, `se_level_changer`, `se_smart_terrain`), so a scene the editor saved without them loads.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::read`] does when the fields are there.
  pub(crate) fn read_or_constructed<R: ChunkFieldReader, V>(
    reader: &mut R,
    owner: &str,
    constructed: impl FnOnce() -> V,
    read: impl FnOnce(&mut R) -> XrfResult<V>,
  ) -> XrfResult<V> {
    if reader.is_field_form() && reader.is_ended() {
      Ok(constructed())
    } else {
      Self::read(reader, owner, read)
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{
    ChunkFieldReader, ChunkFieldReplay, ChunkFieldWriter, ChunkReader, ChunkWriter, InMemoryChunkDataSource,
  };
  use xrf_error::XrfResult;

  use crate::data::alife::zeroed_update_fields::ZeroedUpdateFields;
  use crate::data::generic::script_save_marker::ScriptSaveMarker;

  #[test]
  fn counts_the_fields_it_closes() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    ChunkFieldWriter::write_u8(&mut writer, 7)?;
    ScriptSaveMarker::write(&mut writer, "test", |writer| {
      ChunkFieldWriter::write_u8(writer, 1)?;
      ChunkFieldWriter::write_w1251_string(writer, "hint")
    })?;

    // The byte before the marker opened is not counted: one flag and a four-letter string with its terminator.
    assert_eq!(writer.buffer[writer.buffer.len() - 2..], [6, 0]);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(ChunkFieldReader::read_u8(&mut reader)?, 7);
    assert_eq!(
      ScriptSaveMarker::read(&mut reader, "test", |reader| {
        Ok((
          ChunkFieldReader::read_u8(reader)?,
          ChunkFieldReader::read_w1251_string(reader)?,
        ))
      })?,
      (1, String::from("hint"))
    );

    Ok(())
  }

  // Packet bytes ending there are short, not the editor's form.
  #[test]
  fn takes_constructed_values_only_where_fields_end() -> XrfResult {
    fn read<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<u8> {
      ScriptSaveMarker::read_or_constructed(reader, "test", || 3, ChunkFieldReader::read_u8)
    }

    assert_eq!(read(&mut ChunkFieldReplay::new(&mut ZeroedUpdateFields))?, 3);

    let mut bytes: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[7])?;

    ChunkFieldReader::read_u8(&mut bytes)?;

    assert!(read(&mut bytes).is_err());

    Ok(())
  }

  #[test]
  fn refuses_a_marker_counting_another_size() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[1, 26, 0])?;
    let error: String = ScriptSaveMarker::read(&mut reader, "test", ChunkFieldReader::read_u8)
      .expect_err("a marker that counts more than its fields")
      .to_string();

    assert!(
      error.contains("save marker counts 26 bytes, its fields hold 1"),
      "{error}"
    );

    Ok(())
  }
}
