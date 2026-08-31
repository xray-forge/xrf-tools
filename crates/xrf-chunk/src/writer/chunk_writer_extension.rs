use std::io::Write;

use byteorder::{ByteOrder, WriteBytesExt};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{encode_string_to_w1251_bytes, to_format_size};

use crate::reader::chunk_line::is_line_terminator;
use crate::{ChunkReadWrite, ChunkReadWriteList, ChunkReadWriteOptional, ChunkWriter};

impl ChunkWriter {
  #[inline]
  pub fn write_xr<T: ByteOrder, W: ChunkReadWrite>(&mut self, writable: &W) -> XrfResult {
    writable.write::<T>(self)
  }

  #[inline]
  pub fn write_xr_optional<T: ByteOrder, W: ChunkReadWriteOptional>(&mut self, writable: Option<&W>) -> XrfResult {
    W::write_optional::<T>(self, writable)
  }

  #[inline]
  pub fn write_xr_list<T: ByteOrder, W: ChunkReadWriteList>(&mut self, list: &[W]) -> XrfResult {
    W::write_list::<T>(self, list)
  }

  /// Write null terminated windows1251 encoded string.
  pub fn write_w1251_string(&mut self, data: &str) -> XrfResult<usize> {
    Ok(self.write(&encode_string_to_w1251_bytes(data)?)? + self.write(&[0u8])?)
  }

  /// Write a windows1251 encoded string closed by the line terminator it was read with.
  ///
  /// The terminator is supplied rather than fixed because the engine accepts any run of `\r` and `\n`, and banks
  /// differ; see [`crate::ChunkReader::read_w1251_line`].
  ///
  /// # Errors
  ///
  /// Returns an error when the terminator is not a non-empty run of `\r` and `\n`, which would write a field no
  /// reader can find the end of.
  pub fn write_w1251_line(&mut self, data: &str, terminator: &str) -> XrfResult<usize> {
    if terminator.is_empty() || !terminator.bytes().all(is_line_terminator) {
      return Err(XrfError::new_invalid_error(format!(
        "Cannot write line '{data}' terminated by '{}', expected a run of CR and LF",
        terminator.escape_debug()
      )));
    }

    Ok(self.write(&encode_string_to_w1251_bytes(data)?)? + self.write(terminator.as_bytes())?)
  }

  /// Write serialized vector into vector, where u32 count N is followed by N u16 entries.
  pub fn write_u16_vector<T: ByteOrder>(&mut self, data: &[u16]) -> XrfResult<usize> {
    let count: u32 = to_format_size(data.len(), "u16 vector length")?;

    self.write_u32::<T>(count)?;

    for it in data {
      self.write_u16::<T>(*it)?;
    }

    Ok(size_of::<u32>() + size_of_val(data))
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::{ChunkWriter, XRayByteOrder};

  #[test]
  fn test_write_w1251_string_empty() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(writer.write_w1251_string("")?, 1, "Expect 1 byte written");
    assert_eq!(writer.buffer, [0], "Expect null terminated empty written");
    assert_eq!(writer.bytes_written(), 1, "Expect 1 byte written");

    Ok(())
  }

  #[test]
  fn test_write_w1251_string_sample() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(writer.write_w1251_string("abc")?, 4, "Expect 4 bytes written");
    assert_eq!(
      writer.buffer,
      [b'a', b'b', b'c', 0],
      "Expect null terminated string written"
    );
    assert_eq!(writer.bytes_written(), 4, "Expect 4 bytes written");

    Ok(())
  }

  #[test]
  fn test_write_w1251_line_sample() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(writer.write_w1251_line("abc", "\r\n")?, 5, "Expect 5 bytes written");
    assert_eq!(
      writer.buffer,
      [b'a', b'b', b'c', b'\r', b'\n'],
      "Expect rn terminated string written"
    );

    Ok(())
  }

  #[test]
  fn writes_the_line_terminator_it_was_given() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(writer.write_w1251_line("abc", "\n")?, 4, "Expect 4 bytes written");
    assert_eq!(writer.buffer, [b'a', b'b', b'c', b'\n'], "Expect n terminated string");

    Ok(())
  }

  #[test]
  fn refuses_to_write_a_line_no_reader_could_find_the_end_of() {
    let mut writer: ChunkWriter = ChunkWriter::new();

    for terminator in ["", " ", "\r\n\0"] {
      let error: String = writer
        .write_w1251_line("abc", terminator)
        .expect_err("expect a terminator that is not CR or LF to be refused")
        .to_string();

      assert!(
        error.contains("expected a run of CR and LF"),
        "Unexpected error: {error}"
      );
    }
  }

  #[test]
  fn test_write_u16_vector_empty() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      writer.write_u16_vector::<XRayByteOrder>(&[])?,
      4,
      "Expect 4 bytes written"
    );
    assert_eq!(writer.buffer, [0, 0, 0, 0], "Expect correct written data");
    assert_eq!(writer.bytes_written(), 4, "Expect 4 bytes written with empty vector");

    Ok(())
  }

  #[test]
  fn test_write_u16_vector_samples() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      writer.write_u16_vector::<XRayByteOrder>(&[1, 2, 3, 4])?,
      12,
      "Expect 12 bytes written"
    );
    assert_eq!(
      writer.buffer,
      [4, 0, 0, 0, 1, 0, 2, 0, 3, 0, 4, 0],
      "Expect correct written data"
    );
    assert_eq!(writer.bytes_written(), 12, "Expect 12 bytes written with empty vector");

    Ok(())
  }
}
