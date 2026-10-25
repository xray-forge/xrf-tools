use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{LineEndings, StagedFile, format_path};

use crate::manifest::extern_export::ExternExport;
use crate::render::ExternFormat;
use crate::render::extern_html_renderer::ExternHtmlRenderer;
use crate::render::extern_xml_renderer::ExternXmlRenderer;

/// The public, stable manifest of names exported to X-Ray configuration data.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ExternManifest {
  pub exports: BTreeMap<String, ExternExport>,
}

impl ExternManifest {
  /// The manifest rendered in a format and terminated with a final newline, at the format's own line endings unless
  /// `line_endings` names others: CRLF for JSON, LF for XML and HTML.
  ///
  /// # Errors
  ///
  /// Returns an error when JSON serialization fails.
  pub fn render(&self, format: ExternFormat, line_endings: Option<LineEndings>) -> XrfResult<String> {
    let content: String = match format {
      ExternFormat::Json => serde_json::to_string_pretty(self)?,
      ExternFormat::Xml => ExternXmlRenderer::render(self),
      ExternFormat::Html => ExternHtmlRenderer::render(self),
    };
    let ending: LineEndings = line_endings.unwrap_or_else(|| format.default_line_endings());

    Ok(ending.apply(&format!("{content}\n")))
  }

  /// Publishes the rendered manifest at `path`, creating the directories above it.
  ///
  /// # Errors
  ///
  /// Returns an error when the manifest cannot be rendered, and an IO error when it cannot be published.
  pub fn write(&self, path: &Path, format: ExternFormat, line_endings: Option<LineEndings>) -> XrfResult {
    let content: String = self.render(format, line_endings)?;

    StagedFile::write_creating_directories(path, content.as_bytes()).map_err(|error| {
      XrfError::new_io_error(
        format!(
          "Failed to write the extern artifact to '{}': {error}",
          format_path(path)
        ),
        error.kind(),
      )
    })
  }

  /// Checks that the artifact at `path` still describes this manifest: a JSON one read back as the same manifest, an
  /// XML or HTML one the same text as a fresh render, whatever its line endings.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the artifact cannot be read, an invalid error when a JSON one does not read as a
  /// manifest, and a verify error when it describes another.
  pub fn verify_artifact(&self, path: &Path, format: ExternFormat, line_endings: Option<LineEndings>) -> XrfResult {
    let existing: String = fs::read_to_string(path)?;
    let is_current: bool = match format {
      ExternFormat::Json => {
        let written: Self = serde_json::from_str(&existing).map_err(|error| {
          XrfError::new_invalid_error(format!(
            "Cannot parse '{}' as an extern JSON manifest: {error}",
            format_path(path)
          ))
        })?;

        written == *self
      }
      ExternFormat::Xml | ExternFormat::Html => {
        LineEndings::normalize(&existing) == LineEndings::normalize(&self.render(format, line_endings)?)
      }
    };

    if is_current {
      Ok(())
    } else {
      Err(XrfError::new_verify_error(format!(
        "Extern {} artifact '{}' does not match the parsed declarations.",
        format.get_label(),
        format_path(path)
      )))
    }
  }
}
