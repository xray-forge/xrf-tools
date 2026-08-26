//! Report shapes more than one command answers with.
//!
//! A command that owns its shape keeps it in its own `<command>/report.rs`. These are the shapes
//! several commands genuinely share: `particle pack` and `spawn pack` do the same kind of thing to
//! different bytes, and reporting that in two near-identical shapes would be the per-command drift
//! the reporting contract exists to remove.
//!
//! Paths are portable display strings. Serializing a `PathBuf` fails outright on a host name that is
//! not valid Unicode, and a report describes a run rather than addressing one.

use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use xrf_report::Status;

/// What reading one representation and writing another produced.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileConversionReport {
  destination: String,
  #[serde(with = "xrf_utils::duration_ms")]
  read_duration: Duration,
  source: String,
  #[serde(with = "xrf_utils::duration_ms")]
  write_duration: Duration,
}

impl FileConversionReport {
  pub fn new(source: &Path, destination: &Path, read_duration: Duration, write_duration: Duration) -> Self {
    Self {
      destination: xrf_utils::to_portable_path_string(destination),
      read_duration,
      source: xrf_utils::to_portable_path_string(source),
      write_duration,
    }
  }
}

/// The verdict a command reading one file reached.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileVerifyReport {
  findings: Vec<String>,
  status: Status,
  subject: String,
}

impl FileVerifyReport {
  pub fn new(subject: &Path, findings: Vec<String>) -> Self {
    Self {
      status: Status::from_is_valid(findings.is_empty()),
      findings,
      subject: xrf_utils::to_portable_path_string(subject),
    }
  }

  pub fn passed(subject: &Path) -> Self {
    Self::new(subject, Vec::new())
  }

  pub fn failed(subject: &Path, finding: String) -> Self {
    Self::new(subject, vec![finding])
  }
}

/// What writing one artifact out of an input produced.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWriteReport {
  is_dry: bool,
  source: String,
  written: Vec<String>,
}

impl FileWriteReport {
  pub fn new(source: &Path, written: Vec<&Path>, is_dry: bool) -> Self {
    Self {
      is_dry,
      source: xrf_utils::to_portable_path_string(source),
      written: written.into_iter().map(xrf_utils::to_portable_path_string).collect(),
    }
  }
}
