use std::error::Error;
use std::io::{Error as IoError, ErrorKind};

use image::ImageError;
use thiserror::Error as ThisError;
use xrf_error_derive::ErrorConstructors;

/// Error produced by XRF tools and libraries.
///
/// Internal to Rust: a boundary carries one as its message. What a surface reports about the data it judged is a
/// finding of that domain's own type, never one of these.
#[derive(ThisError, Debug, ErrorConstructors)]
pub enum XrfError {
  #[constructor]
  #[error("Assertion error: {message}")]
  Assertion { message: String },
  #[constructor]
  #[error("Asset error: {message}")]
  Asset { message: String },
  #[constructor]
  #[error("Convert error: {message}")]
  Convert { message: String },
  #[constructor]
  #[error("Verify error: {message}")]
  Verify { message: String },
  #[constructor]
  #[error("Not implemented error: {message}")]
  NotImplemented { message: String },
  #[constructor]
  #[error("Read error: {message}")]
  Read { message: String },
  #[constructor]
  #[error("Unexpected error: {message}")]
  Unexpected { message: String },
  #[constructor]
  #[error("Not found error: {message}")]
  NotFound { message: String },
  #[constructor]
  #[error("Invalid error: {message}")]
  Invalid { message: String },
  #[constructor]
  #[error("Parsing error: {message}")]
  Parsing { message: String },
  #[constructor]
  #[error("Encoding error: {message}")]
  Encoding { message: String },
  #[constructor]
  #[error("Missing terminator error: {message}")]
  NoTerminator { message: String },
  #[constructor]
  #[error("Unknown language: {message}")]
  UnknownLanguage { message: String },
  #[constructor]
  #[error("Serialization error: {message}")]
  Serialization { message: String },
  #[constructor]
  #[error("Texture processing error: {message}")]
  TextureProcessing { message: String },
  #[error("Chunk is not ended, {remaining} bytes remain: {message}")]
  ChunkNotEnded { message: String, remaining: u64 },
  #[error("Ltx parse error: {line}:{col} {message}")]
  LtxParse { line: usize, col: usize, message: String },
  /// A scheme declaration that cannot be read as one: a defect in the scheme files, not in the configs they judge.
  #[error("Ltx scheme error [{section}] {field} : {message}")]
  LtxScheme {
    section: String,
    field: String,
    message: String,
  },
  /// An operation stopped at a safe boundary because it was asked to.
  ///
  /// Control flow rather than a failure: it exists so a cancellation check composes with `?` and can break a parallel
  /// iterator, which stops on an error and on nothing else. An operation is expected to catch its own and report what
  /// it completed, so this reaching a caller means one forgot to.
  #[constructor]
  #[error("Cancelled: {message}")]
  Cancelled { message: String },
  #[constructor]
  #[error("Generic error: {message}")]
  Generic { message: String },
  #[constructor]
  #[error("Serde error: {message}")]
  Serde { message: String },
  #[error("IO error: {message}")]
  Io { message: String, kind: ErrorKind },
}

impl XrfError {
  pub fn new_chunk_not_ended_error<T>(message: T, remaining: u64) -> Self
  where
    T: Into<String>,
  {
    Self::ChunkNotEnded {
      message: message.into(),
      remaining,
    }
  }

  pub fn new_ltx_parse_error<T>(line: usize, col: usize, message: T) -> Self
  where
    T: Into<String>,
  {
    Self::LtxParse {
      line,
      col,
      message: message.into(),
    }
  }

  pub fn new_ltx_scheme_error<S, F, M>(section: S, field: F, message: M) -> Self
  where
    S: Into<String>,
    F: Into<String>,
    M: Into<String>,
  {
    Self::LtxScheme {
      section: section.into(),
      field: field.into(),
      message: message.into(),
    }
  }

  pub fn new_io_error<T>(message: T, kind: ErrorKind) -> Self
  where
    T: Into<String>,
  {
    Self::Io {
      message: message.into(),
      kind,
    }
  }
}

impl From<IoError> for XrfError {
  fn from(value: IoError) -> Self {
    Self::Io {
      message: value.to_string(),
      kind: value.kind(),
    }
  }
}

impl From<serde_json::error::Error> for XrfError {
  fn from(value: serde_json::error::Error) -> Self {
    Self::Serde {
      message: value.to_string(),
    }
  }
}

impl From<ImageError> for XrfError {
  fn from(value: ImageError) -> Self {
    Self::TextureProcessing {
      message: value.to_string(),
    }
  }
}

impl From<Box<dyn Error + Send + Sync>> for XrfError {
  fn from(value: Box<dyn Error + Send + Sync>) -> Self {
    Self::Generic {
      message: value.to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io::{Error as IoError, ErrorKind};

  use super::XrfError;

  #[test]
  fn formats_ltx_scheme_declaration_errors_by_section_and_field() {
    assert_eq!(
      XrfError::new_ltx_scheme_error("section", "field", "message").to_string(),
      "Ltx scheme error [section] field : message"
    );
  }

  #[test]
  fn io_conversion_preserves_kind_beside_the_message() {
    let error: XrfError = IoError::new(ErrorKind::NotFound, "missing file").into();

    assert!(matches!(
      &error,
      XrfError::Io {
        message,
        kind: ErrorKind::NotFound,
      } if message == "missing file"
    ));
    assert_eq!(error.to_string(), "IO error: missing file");
  }

  #[test]
  fn converts_serde_and_boxed_errors_to_their_variants() {
    let serde: XrfError = serde_json::from_str::<u32>("x").expect_err("not a number").into();
    let boxed: XrfError = Box::<dyn std::error::Error + Send + Sync>::from("boxed").into();

    assert!(matches!(serde, XrfError::Serde { .. }));
    assert!(matches!(&boxed, XrfError::Generic { message } if message == "boxed"));
  }
}
