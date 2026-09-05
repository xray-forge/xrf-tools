use std::io::ErrorKind;

use serde::Serialize;
use thiserror::Error as ThisError;
use xrf_error_derive::ErrorConstructors;

/// Error produced by XRF tools and libraries.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(ThisError, Debug, ErrorConstructors, Serialize)]
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
  #[error("Ltx format error: {message}")]
  Format { message: String },
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
  #[error("Invalid source: {message}")]
  InvalidSource { message: String },
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
  #[error(
  "Ltx scheme error{location} [{section}] {field} : {message}",
  location = XrfError::format_ltx_scheme_location(at.as_deref(), entry.as_deref())
)]
  LtxScheme {
    section: String,
    field: String,
    message: String,
    /// The config to open: the one that declared the section, where a dialect said which.
    at: Option<String>,
    /// The entry point whose resolution reached the section, when that is a different file.
    ///
    /// Kept beside `at` rather than instead of it: the entry point is the unit a caller re-runs and the only thing
    /// that explains why two files were read together, but it is not where the section is written.
    entry: Option<String>,
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
  Io {
    message: String,
    // The source error is diagnostic context and is intentionally absent from the wire contract.
    #[serde(skip_serializing)]
    #[cfg_attr(feature = "typescript-bindings", specta(skip))]
    kind: ErrorKind,
  },
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
      entry: None,
      field: field.into(),
      message: message.into(),
      at: None,
    }
  }

  pub fn new_scheme_error_at<S, F, M, A>(section: S, field: F, message: M, at: A) -> Self
  where
    S: Into<String>,
    F: Into<String>,
    M: Into<String>,
    A: Into<String>,
  {
    Self::LtxScheme {
      section: section.into(),
      entry: None,
      field: field.into(),
      message: message.into(),
      at: Some(at.into()),
    }
  }

  /// A scheme finding at the config that declared the section, resolved through an entry point.
  ///
  /// `declared_in` is `None` where the dialect could not say - an override that created a section nothing declares -
  /// and the finding then names the entry point alone rather than implying an origin it does not know.
  pub fn new_scheme_error_resolved<S, F, M>(
    section: S,
    field: F,
    message: M,
    declared_in: Option<&str>,
    resolved_from: &str,
  ) -> Self
  where
    S: Into<String>,
    F: Into<String>,
    M: Into<String>,
  {
    Self::LtxScheme {
      section: section.into(),
      entry: Some(String::from(resolved_from)),
      field: field.into(),
      message: message.into(),
      at: declared_in.map(String::from),
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

impl XrfError {
  fn format_ltx_scheme_location(at: Option<&str>, entry: Option<&str>) -> String {
    match (at, entry) {
      (Some(at), Some(entry)) if at != entry => format!(" in '{at}' resolved from '{entry}'"),
      (Some(path), _) | (None, Some(path)) => format!(" in '{path}'"),
      (None, None) => String::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::io::{Error as IoError, ErrorKind};

  use serde_json::json;

  use super::XrfError;

  #[test]
  fn formats_ltx_scheme_error_locations_readably() {
    assert_eq!(
      XrfError::new_scheme_error_at("section", "field", "message", "configs/system.ltx").to_string(),
      "Ltx scheme error in 'configs/system.ltx' [section] field : message"
    );
    assert_eq!(
      XrfError::new_scheme_error_resolved(
        "section",
        "field",
        "message",
        Some("configs/items/w_broken.ltx"),
        "configs/system.ltx"
      )
      .to_string(),
      "Ltx scheme error in 'configs/items/w_broken.ltx' resolved from 'configs/system.ltx' [section] field : message"
    );
    // A section written in the entry point itself reads as it always did, rather than naming one file twice.
    assert_eq!(
      XrfError::new_scheme_error_resolved(
        "section",
        "field",
        "message",
        Some("configs/system.ltx"),
        "configs/system.ltx"
      )
      .to_string(),
      "Ltx scheme error in 'configs/system.ltx' [section] field : message"
    );
    // Nothing said where it was declared, so the entry point is named rather than an origin that is not known.
    assert_eq!(
      XrfError::new_scheme_error_resolved("section", "field", "message", None, "configs/system.ltx").to_string(),
      "Ltx scheme error in 'configs/system.ltx' [section] field : message"
    );
    assert_eq!(
      XrfError::new_ltx_scheme_error("section", "field", "message").to_string(),
      "Ltx scheme error [section] field : message"
    );
  }

  #[test]
  fn preserves_externally_tagged_serialization_contract() {
    let error = XrfError::new_scheme_error_at("section", "field", "message", "configs/system.ltx");

    assert_eq!(
      serde_json::to_value(error).expect("XRF errors should serialize"),
      json!({
        "LtxScheme": {
          "section": "section",
          "field": "field",
          "message": "message",
          "at": "configs/system.ltx",
          "entry": null
        }
      })
    );

    // A finding whose section was declared somewhere other than the entry point that resolved it carries both, so a
    // report says which file to open without losing the unit a caller re-runs.
    assert_eq!(
      serde_json::to_value(XrfError::new_scheme_error_resolved(
        "section",
        "field",
        "message",
        Some("configs/items/w_broken.ltx"),
        "configs/system.ltx",
      ))
      .expect("XRF errors should serialize"),
      json!({
        "LtxScheme": {
          "section": "section",
          "field": "field",
          "message": "message",
          "at": "configs/items/w_broken.ltx",
          "entry": "configs/system.ltx"
        }
      })
    );
  }

  #[test]
  fn io_conversion_preserves_kind_without_exposing_it_on_the_wire() {
    let error: XrfError = IoError::new(ErrorKind::NotFound, "missing file").into();

    assert!(matches!(
      &error,
      XrfError::Io {
        message,
        kind: ErrorKind::NotFound,
      } if message == "missing file"
    ));
    assert_eq!(error.to_string(), "IO error: missing file");
    assert_eq!(
      serde_json::to_value(error).expect("XRF errors should serialize"),
      json!({
        "Io": {
          "message": "missing file"
        }
      })
    );
  }
}
