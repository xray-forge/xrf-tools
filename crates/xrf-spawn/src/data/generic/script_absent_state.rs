use std::fmt::Debug;

use byteorder::ByteOrder;
use xrf_chunk::ChunkFieldReader;
use xrf_error::{XrfError, XrfResult};

/// A script class's field that only a running simulation fills: a list's count, a time's `nil` mark, a nested object's
/// `nil` name. A spawn holds its empty form, which is read and checked and has no field of its own; any other value is
/// refused, since the records behind it are not read and a rewrite would drop them.
pub(crate) struct ScriptAbsentState;

impl ScriptAbsentState {
  /// Read a byte `field` of `owner`, refusing any value but `absent`.
  ///
  /// # Errors
  ///
  /// Returns an error when the reader fails or the byte is not `absent`.
  pub(crate) fn read_u8<R: ChunkFieldReader>(reader: &mut R, owner: &str, field: &str, absent: u8) -> XrfResult {
    Self::assert_absent(owner, field, &reader.read_u8()?, &absent)
  }

  /// Read a `u16` `field` of `owner`, refusing any value but `absent`.
  ///
  /// # Errors
  ///
  /// Returns an error when the reader fails or the value is not `absent`.
  pub(crate) fn read_u16<T: ByteOrder, R: ChunkFieldReader>(
    reader: &mut R,
    owner: &str,
    field: &str,
    absent: u16,
  ) -> XrfResult {
    Self::assert_absent(owner, field, &reader.read_u16::<T>()?, &absent)
  }

  /// Read a string `field` of `owner`, refusing any text but `absent`.
  ///
  /// # Errors
  ///
  /// Returns an error when the reader fails or the text is not `absent`.
  pub(crate) fn read_string<R: ChunkFieldReader>(reader: &mut R, owner: &str, field: &str, absent: &str) -> XrfResult {
    Self::assert_absent(owner, field, &reader.read_w1251_string()?.as_str(), &absent)
  }

  fn assert_absent<V: Debug + PartialEq>(owner: &str, field: &str, value: &V, absent: &V) -> XrfResult {
    if value == absent {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "{owner} holds {field} {value:?} where a spawn holds {absent:?}: simulation state that is not read"
    )))
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::data::generic::script_absent_state::ScriptAbsentState;

  #[test]
  fn reads_the_empty_form_and_refuses_another() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(b"\x00\xff\xff\x00nil\0")?;

    ScriptAbsentState::read_u8(&mut reader, "test", "count", 0)?;
    ScriptAbsentState::read_u16::<XRayByteOrder, _>(&mut reader, "test", "id", u16::MAX)?;

    let error: String = ScriptAbsentState::read_u8(&mut reader, "se_test", "squads count", 3)
      .expect_err("a count the spawn does not hold")
      .to_string();

    assert_eq!(
      error,
      "Invalid error: se_test holds squads count 0 where a spawn holds 3: simulation state that is not read"
    );

    ScriptAbsentState::read_string(&mut reader, "test", "game", "nil")?;
    assert!(reader.is_ended());

    Ok(())
  }
}
