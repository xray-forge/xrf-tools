use std::fmt::Display;

use crate::xrf_error::XrfError;
use crate::xrf_result::XrfResult;

/// Maps another library's error to the [`XrfError`] variant that names its cause, keeping its `Display` text.
///
/// Call it only on another library's error. [`XrfError`] is `Display` too, so on an [`XrfResult`] it compiles and
/// replaces the variant a caller would match on.
pub trait MapXrfErr<T> {
  /// Keeps the error's `Display` text as a `TextureProcessing` error.
  fn map_texture_processing_err(self) -> XrfResult<T>;

  /// Keeps the error's `Display` text as a `Serde` error.
  fn map_serde_err(self) -> XrfResult<T>;
}

impl<T, E: Display> MapXrfErr<T> for Result<T, E> {
  fn map_texture_processing_err(self) -> XrfResult<T> {
    self.map_err(|error| XrfError::new_texture_processing_error(error.to_string()))
  }

  fn map_serde_err(self) -> XrfResult<T> {
    self.map_err(|error| XrfError::new_serde_error(error.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use super::MapXrfErr;
  use crate::xrf_error::XrfError;

  #[test]
  fn maps_a_foreign_error_to_the_named_variant_keeping_its_text() {
    let texture = Err::<(), _>("bad pixel").map_texture_processing_err();
    let serde = Err::<(), _>("bad json").map_serde_err();

    assert!(matches!(&texture, Err(XrfError::TextureProcessing { message }) if message == "bad pixel"));
    assert!(matches!(&serde, Err(XrfError::Serde { message }) if message == "bad json"));
    assert_eq!(Ok::<u8, &str>(1).map_serde_err().ok(), Some(1));
  }
}
