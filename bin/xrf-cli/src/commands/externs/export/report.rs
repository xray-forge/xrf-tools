use std::path::Path;

use serde::Serialize;
use xrf_export::ExternFormat;
use xrf_report::Status;

/// What `externs export` rendered, or judged an existing artifact against.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternsExportReport {
  destination: String,
  externs: usize,
  findings: Vec<String>,
  format: String,
  is_check: bool,
  source: String,
  status: Status,
}

impl ExternsExportReport {
  /// What a `--check` run compared, and the mismatch it found, if any.
  pub fn checked(
    source: &Path,
    destination: &Path,
    format: ExternFormat,
    externs: usize,
    findings: Vec<String>,
  ) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      externs,
      format: String::from(format_name(format)),
      is_check: true,
      source: xrf_utils::to_portable_path_string(source),
      status: Status::from_is_valid(findings.is_empty()),
      findings,
    }
  }

  /// What a writing run rendered, and where it put it.
  pub fn written(source: &Path, destination: &Path, format: ExternFormat, externs: usize) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      externs,
      findings: Vec::new(),
      format: String::from(format_name(format)),
      is_check: false,
      source: xrf_utils::to_portable_path_string(source),
      status: Status::Passed,
    }
  }
}

/// The format under the name `--format` accepts, so a report round-trips into a repeat invocation.
const fn format_name(format: ExternFormat) -> &'static str {
  match format {
    ExternFormat::Html => "html",
    ExternFormat::Json => "json",
    ExternFormat::Xml => "xml",
  }
}
