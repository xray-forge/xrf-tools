use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::narrow_format_size;

/// The chunk handling every container of an editable object shares: one framed child at a time on write, and on read
/// each child either folded into a field or kept as it was stored.
pub(crate) struct EditableChunks;

impl EditableChunks {
  /// Frames what `write` puts in one child chunk and appends it to `destination`.
  pub(crate) fn write<T: ByteOrder>(
    destination: &mut ChunkWriter,
    id: u32,
    write: impl FnOnce(&mut ChunkWriter) -> XrfResult,
  ) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    write(&mut writer)?;
    writer.flush_chunk_into::<T>(destination, id)?;

    Ok(())
  }

  /// Frames a run of children numbered from zero, which is how the SDK stores a list of meshes or bones: it opens
  /// child 0, 1, 2 and stops at the first number missing.
  pub(crate) fn write_numbered<T: ByteOrder, I>(
    destination: &mut ChunkWriter,
    items: &[I],
    write: impl Fn(&I, &mut ChunkWriter) -> XrfResult,
  ) -> XrfResult {
    for (index, item) in items.iter().enumerate() {
      Self::write::<T>(destination, narrow_format_size(index, "Numbered chunk")?, |writer| {
        write(item, writer)
      })?;
    }

    Ok(())
  }

  /// Reads a run of children numbered from zero, refusing any other numbering: the SDK reads such a list only up to
  /// the first number missing, so a gap would hide what follows it from the editor while this reader kept it.
  pub(crate) fn read_numbered<D: ChunkDataSource, I>(
    reader: &mut ChunkReader<D>,
    what: &str,
    mut read: impl FnMut(&mut ChunkReader<D>) -> XrfResult<I>,
  ) -> XrfResult<Vec<I>> {
    let children: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut items: Vec<I> = Vec::with_capacity(children.len());

    for (index, mut child) in children.into_iter().enumerate() {
      if u64::from(child.id) != index as u64 {
        return Err(XrfError::new_parsing_error(format!(
          "Unexpected {what} chunk {} at position {index}, the editor numbers them from 0 without gaps",
          child.id
        )));
      }

      items.push(read(&mut child)?);
    }

    Ok(items)
  }

  /// Reads a chunk folded into a field and proves the field was all of it, so no byte is dropped on the way back.
  pub(crate) fn read_whole<D: ChunkDataSource, V>(
    chunk: &mut ChunkReader<D>,
    what: &str,
    read: impl FnOnce(&mut ChunkReader<D>) -> XrfResult<V>,
  ) -> XrfResult<V> {
    let value: V = read(chunk)?;

    chunk.assert_read(&format!("Expect all data to be read from {what} chunk"))?;

    Ok(value)
  }

  /// Reads a `u32` count and that many records, bounding the count by the bytes left before reserving for it.
  pub(crate) fn read_counted<T: ByteOrder, D: ChunkDataSource, R>(
    chunk: &mut ChunkReader<D>,
    min_record_size: u64,
    what: &str,
    read: impl Fn(&mut ChunkReader<D>) -> XrfResult<R>,
  ) -> XrfResult<Vec<R>> {
    let count: u32 = chunk.read_u32::<T>()?;
    let mut records: Vec<R> = chunk.new_bounded_vec(count.into(), min_record_size, what)?;

    for _ in 0..count {
      records.push(read(chunk)?);
    }

    Ok(records)
  }

  /// Writes a `u32` count and the records.
  pub(crate) fn write_counted<T: ByteOrder, R: ChunkReadWrite>(
    writer: &mut ChunkWriter,
    records: &[R],
    what: &str,
  ) -> XrfResult {
    writer.write_u32::<T>(narrow_format_size(records.len(), what)?)?;

    for record in records {
      record.write::<T>(writer)?;
    }

    Ok(())
  }

  /// Fails with the SDK's own complaint when a chunk it requires is missing.
  pub(crate) fn require<V>(value: Option<V>, what: &str) -> XrfResult<V> {
    value.ok_or_else(|| XrfError::new_parsing_error(format!("Editable object carries no {what} chunk")))
  }

  /// Refuses a version the SDK refuses to load.
  pub(crate) fn require_version(version: u16, expected: u16, what: &str) -> XrfResult {
    if version == expected {
      Ok(())
    } else {
      Err(XrfError::new_not_implemented_error(format!(
        "Unexpected {what} version {version}, the SDK loads only version {expected}"
      )))
    }
  }
}
