use std::io::Write;

use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{encode_string_to_w1251_bytes, encode_w1251_bytes_to_string};

/// Reads a fixed width, null terminated windows-1251 field.
///
/// # Errors
///
/// When the chunk holds fewer than `size` bytes, when no terminator is inside the field, or when the bytes are not
/// decodable.
pub(crate) fn read_shader_fixed_string<D: ChunkDataSource>(
  reader: &mut ChunkReader<D>,
  size: usize,
  what: &str,
) -> XrfResult<String> {
  let bytes: Vec<u8> = reader.read_bytes(size)?;
  let Some(end) = bytes.iter().position(|byte| *byte == 0) else {
    return Err(XrfError::new_no_terminator_error(format!(
      "Shader {what} is not null terminated"
    )));
  };

  Ok(encode_w1251_bytes_to_string(&bytes[..end])?)
}

/// Writes a fixed width field, null padded the way the engine's own buffers are.
///
/// # Errors
///
/// When the value leaves no room for its terminator, which is the length the engine's own buffer would truncate at.
pub(crate) fn write_shader_fixed_string(writer: &mut ChunkWriter, value: &str, size: usize, what: &str) -> XrfResult {
  let bytes: Vec<u8> = encode_string_to_w1251_bytes(value)?;

  if bytes.len() >= size {
    return Err(XrfError::new_invalid_error(format!(
      "Shader {what} '{value}' does not fit in {size} bytes"
    )));
  }

  let mut buffer: Vec<u8> = vec![0; size];

  buffer[..bytes.len()].copy_from_slice(&bytes);

  writer.write_all(&buffer)?;

  Ok(())
}
