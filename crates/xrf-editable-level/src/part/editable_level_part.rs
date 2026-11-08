use std::path::Path;

use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{StagedFile, format_path};

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::part::editable_binary_part::EditableBinaryPart;
use crate::part::editable_text_part::EditableTextPart;

/// One file of a scene's part directory, `<tool>.part` or a numbered `<tool>.partN`, holding one tool's objects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditableLevelPart {
  Text(EditableTextPart),
  Binary(EditableBinaryPart),
}

impl EditableLevelPart {
  /// The extension of a tool's first part file; the next ones append their number to it.
  pub const EXTENSION: &str = "part";

  /// Whether a file name is one the editor reads as a tool's part, `<tool>.part` or `<tool>.partN`, rather than a
  /// `.~part` copy or another file.
  pub fn is_part_file_name(file_name: &str) -> bool {
    let Some((tool, extension)) = file_name.rsplit_once('.') else {
      return false;
    };

    !tool.is_empty()
      && extension
        .strip_prefix(Self::EXTENSION)
        .is_some_and(|number| number.bytes().all(|byte| byte.is_ascii_digit()))
  }

  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a part [`Self::read_from_bytes`] reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    let bytes: Vec<u8> = std::fs::read(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level part was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?;

    Self::read_from_bytes::<T>(bytes)
  }

  /// Reads a part as `EScene::LoadLevelPartLTX` tells the two kinds apart: a file opening with `[` is text, any other
  /// is chunks.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not the part their first byte says they are.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    if bytes.first() == Some(&b'[') {
      Ok(Self::Text(EditableTextPart::read_from_bytes(&bytes)?))
    } else {
      Ok(Self::Binary(EditableBinaryPart::read_from_bytes::<T>(bytes)?))
    }
  }

  /// Writes the part through a staged file, creating the directories leading to it.
  ///
  /// # Errors
  ///
  /// Returns an error as [`Self::write_to_bytes`] does, or when the file cannot be written.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    Ok(StagedFile::write_creating_directories(
      path.as_ref(),
      &self.write_to_bytes::<T>()?,
    )?)
  }

  /// # Errors
  ///
  /// Returns an error when a text part would not read back as itself or a binary part's chunks cannot be framed.
  pub fn write_to_bytes<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    match self {
      Self::Text(part) => part.write_to_bytes(),
      Self::Binary(part) => part.write_to_bytes::<T>(),
    }
  }

  /// # Errors
  ///
  /// Returns an error when a text part's `[guid]` is missing or malformed.
  pub fn get_guid(&self) -> XrfResult<EditableLevelGuid> {
    match self {
      Self::Text(part) => part.get_guid(),
      Self::Binary(part) => Ok(part.guid),
    }
  }
}
