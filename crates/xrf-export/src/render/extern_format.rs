use std::path::Path;
use std::str::FromStr;

use xrf_error::{XrfError, XrfResult};
use xrf_extension::{XrayExtension, XrayExtensionOf};
use xrf_utils::{LineEndings, format_path};

/// A format the manifest is published in, as `xrf-cli externs export --format` names it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternFormat {
  Json,
  Xml,
  Html,
}

impl ExternFormat {
  /// Every name `--format` accepts, in the order the flag lists them.
  pub const NAMES: [&'static str; 3] = [Self::Json.as_str(), Self::Xml.as_str(), Self::Html.as_str()];

  /// The name this format is asked for and reported by, which is the extension it is normally written with.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Json => XrayExtension::Json.as_str(),
      Self::Xml => XrayExtension::Xml.as_str(),
      Self::Html => XrayExtension::Html.as_str(),
    }
  }

  /// How messages name the format.
  pub const fn get_label(self) -> &'static str {
    match self {
      Self::Json => "JSON",
      Self::Xml => "XML",
      Self::Html => "HTML",
    }
  }

  /// Returns the repository default line ending for this output format.
  pub fn default_line_endings(self) -> LineEndings {
    match self {
      Self::Json => LineEndings::Crlf,
      Self::Xml | Self::Html => LineEndings::Lf,
    }
  }

  /// Infers a format from a `.json`, `.xml`, `.html`, or `.htm` extension.
  ///
  /// Two spellings answer HTML because both are what a person writes; which of them a file carries says nothing
  /// about its contents.
  ///
  /// # Errors
  ///
  /// Returns an error when the path has no supported extension.
  pub fn from_extension(path: &Path) -> XrfResult<Self> {
    match XrayExtensionOf::of_path(path).known() {
      Some(XrayExtension::Json) => Ok(Self::Json),
      Some(XrayExtension::Xml) => Ok(Self::Xml),
      Some(XrayExtension::Html | XrayExtension::Htm) => Ok(Self::Html),
      _ => Err(XrfError::new_invalid_error(format!(
        "Cannot infer extern export format from '{}'; use --format.",
        format_path(path)
      ))),
    }
  }
}

impl FromStr for ExternFormat {
  type Err = XrfError;

  /// Selects a format by the name `--format` was given.
  fn from_str(value: &str) -> XrfResult<Self> {
    match XrayExtension::parse(value) {
      Some(XrayExtension::Json) => Ok(Self::Json),
      Some(XrayExtension::Xml) => Ok(Self::Xml),
      Some(XrayExtension::Html) => Ok(Self::Html),
      _ => Err(XrfError::new_invalid_error(format!(
        "Unsupported extern export format '{value}'. Expected {}, {}, or {}.",
        Self::Json.as_str(),
        Self::Xml.as_str(),
        Self::Html.as_str()
      ))),
    }
  }
}
