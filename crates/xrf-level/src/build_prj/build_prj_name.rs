use std::io::Write;

use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{decode_w1251_bytes_to_string, encode_string_to_w1251_bytes};

/// A name in one of the editor's fixed buffers, `string128` or `string64`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjName {
  pub value: String,
  /// The buffer from the terminator to its end, terminator included: the editor blits the whole buffer, so whatever a
  /// longer name left there follows, and keeping it is what makes a rewrite give back the file it read.
  pub trailing: Vec<u8>,
}

impl BuildPrjName {
  /// Bytes a `string128` field occupies: shader and texture names.
  pub const LONG_SIZE: usize = 128;

  /// Bytes a `string64` field occupies: light control names.
  pub const SHORT_SIZE: usize = 64;

  /// A name padded with zeros, which is what a fresh buffer holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the name is not Windows-1251, or leaves no room for its terminator.
  pub fn new(value: &str, size: usize) -> XrfResult<Self> {
    let length: usize = encode_string_to_w1251_bytes(value)?.len();

    if length >= size {
      return Err(XrfError::new_invalid_error(format!(
        "Name '{value}' leaves no room for its terminator in {size} bytes"
      )));
    }

    Ok(Self {
      value: String::from(value),
      trailing: vec![0; size - length],
    })
  }

  /// Reads a field `size` bytes wide.
  ///
  /// # Errors
  ///
  /// Returns an error when fewer than `size` bytes remain, the field holds no terminator, or the name is not
  /// Windows-1251.
  pub fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>, size: usize) -> XrfResult<Self> {
    let bytes: Vec<u8> = reader.read_bytes(size)?;
    let Some(end) = bytes.iter().position(|byte| *byte == 0) else {
      return Err(XrfError::new_no_terminator_error(format!(
        "A {size}-byte build project name is not null terminated"
      )));
    };

    Ok(Self {
      value: decode_w1251_bytes_to_string(&bytes[..end])?,
      trailing: bytes[end..].to_vec(),
    })
  }

  /// Writes the field back, `size` bytes wide.
  ///
  /// # Errors
  ///
  /// Returns an error when the name and its trailing bytes do not fill the field exactly, which means one was changed
  /// without the other, or the writer refuses the bytes.
  pub fn write(&self, writer: &mut ChunkWriter, size: usize) -> XrfResult {
    let mut bytes: Vec<u8> = encode_string_to_w1251_bytes(&self.value)?;

    bytes.extend_from_slice(&self.trailing);

    if bytes.len() != size || self.trailing.first() != Some(&0) {
      return Err(XrfError::new_invalid_error(format!(
        "Name '{}' and its {} trailing bytes do not fill the {size}-byte field holding it",
        self.value,
        self.trailing.len()
      )));
    }

    writer.write_all(&bytes)?;

    Ok(())
  }
}
