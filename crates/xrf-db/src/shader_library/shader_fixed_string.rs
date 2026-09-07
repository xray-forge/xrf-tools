use std::io::Write;

use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{encode_string_to_w1251_bytes, encode_w1251_bytes_to_string};

/// Reads a fixed width, null terminated windows-1251 field.
///
/// The shader library stores every name in a buffer of its own width - `string64` for a property value, `string128`
/// for a blender's name, `string32` for the machine that saved it - and writes the whole buffer whatever the name
/// inside it is. The field is therefore consumed by width and the value ends at its terminator; what an authoring tool
/// left in the rest of the buffer is not part of it.
///
/// A field with no terminator is refused rather than taken whole. Every engine and SDK writer reaches these buffers
/// through `xr_strcpy`, which always terminates, so a buffer that runs to its end is a record that is not a record -
/// and inventing a name from it would hide that.
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
