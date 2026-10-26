use std::io::Write;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::chunks::thm_extra_chunk::ThmExtraChunk;
use crate::chunks::thm_trailing_bytes::ThmTrailingBytes;

/// How every thumbnail kind reads and writes its chunks, so that each writes back the file it was read from: a chunk
/// read as a field keeps the bytes past its fields, and a chunk read as none is kept whole.
pub(crate) struct ThmChunks;

impl ThmChunks {
  /// Keeps what `chunk` holds past what its reader took: whole in `extra` when nothing of it was folded into a field,
  /// otherwise as `trailing` bytes after the fields.
  ///
  /// The SDK's readers take a chunk's fields and nothing past them, and some writers leave more: a type chunk a byte
  /// longer, in some texture packs.
  pub(crate) fn keep_rest<D: ChunkDataSource>(
    chunk: &mut ChunkReader<D>,
    is_folded: bool,
    extra: &mut Vec<ThmExtraChunk>,
    trailing: &mut Vec<ThmTrailingBytes>,
  ) -> XrfResult {
    let rest: Vec<u8> = chunk.read_remaining()?;

    if !is_folded {
      extra.push(ThmExtraChunk {
        id: chunk.id,
        data: rest,
      });
    } else if !rest.is_empty() {
      trailing.push(ThmTrailingBytes {
        id: chunk.id,
        data: rest,
      });
    }

    Ok(())
  }

  /// Frames a folded chunk's fields as a chunk of `id`, then the bytes `trailing` kept past them.
  pub(crate) fn write<T: ByteOrder>(
    destination: &mut dyn Write,
    id: u32,
    trailing: &[ThmTrailingBytes],
    fill: impl FnOnce(&mut ChunkWriter) -> XrfResult,
  ) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    fill(&mut writer)?;

    for kept in trailing.iter().filter(|kept| kept.id == id) {
      writer.write_all(&kept.data)?;
    }

    writer.flush_chunk_into::<T>(destination, id)?;

    Ok(())
  }

  /// Checks a thumbnail declares the version and type its kind's reader accepts, as the SDK's `Load` refuses another
  /// version and asserts on another type.
  ///
  /// # Errors
  ///
  /// Returns an error naming `subject` when either is missing or not the one expected.
  pub(crate) fn require_kind(
    subject: &str,
    version: Option<u16>,
    expected_version: u16,
    thumbnail_type: Option<u32>,
    expected_type: u32,
  ) -> XrfResult {
    if version != Some(expected_version) {
      return Err(XrfError::new_parsing_error(format!(
        "Thumbnail version {version:?} is not {expected_version:#06x}, the only one the SDK reads for {subject}"
      )));
    }

    if thumbnail_type != Some(expected_type) {
      return Err(XrfError::new_parsing_error(format!(
        "Thumbnail of type {thumbnail_type:?} is not {subject}'s, which is {expected_type}"
      )));
    }

    Ok(())
  }

  /// Writes every chunk no field holds, after the folded ones, as it was read.
  pub(crate) fn write_extra<T: ByteOrder>(destination: &mut dyn Write, extra: &[ThmExtraChunk]) -> XrfResult {
    for chunk in extra {
      Self::write::<T>(destination, chunk.id, &[], |writer| chunk.write(writer))?;
    }

    Ok(())
  }
}
