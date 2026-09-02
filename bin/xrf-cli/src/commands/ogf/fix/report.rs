use std::path::Path;

use serde::Serialize;
use xrf_db::OgfResidue;

/// One visual whose bytes `ogf fix` changed, or would change on a dry run.
///
/// Listed per file because the discarded reference is the reason the command reports at all: after the write, this and
/// the console line are the only record of what the source carried.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixFileReport {
  /// Why the residue was inert to the engine: `split-motion-ref` or `trailing-fragment`. Absent when the only discarded
  /// bytes sat inside the motion refs chunk, past its count.
  cause: Option<String>,
  destination: String,
  /// The motion reference path the residue completed, for the split-reference shape only.
  discarded_reference: Option<String>,
  /// Bytes the source carried that the engine never read, inside the motion refs chunk and after it.
  pub discarded_size: usize,
  is_written: bool,
  normalized_size: usize,
  original_size: usize,
  source: String,
}

impl OgfFixFileReport {
  pub fn new(
    source: &Path,
    destination: &Path,
    original_size: usize,
    normalized_size: usize,
    residue: Option<&OgfResidue>,
    is_written: bool,
  ) -> Self {
    Self {
      cause: residue.map(|residue| String::from(residue.cause.as_str())),
      destination: xrf_utils::to_portable_path_string(destination),
      discarded_reference: residue
        .and_then(|residue| residue.cause.get_discarded_path())
        .map(String::from),
      discarded_size: original_size.saturating_sub(normalized_size),
      is_written,
      normalized_size,
      original_size,
      source: xrf_utils::to_portable_path_string(source),
    }
  }
}

/// One visual the command could not fix: unreadable, refused by the reader, or not written.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixFindingReport {
  message: String,
  source: String,
}

impl OgfFixFindingReport {
  pub fn new(source: &Path, message: String) -> Self {
    Self {
      message,
      source: xrf_utils::to_portable_path_string(source),
    }
  }
}

/// How one visual came out of the run.
pub enum OgfFixOutcome {
  Unchanged,
  Normalized(OgfFixFileReport),
  Failed(OgfFixFindingReport),
}

/// What `ogf fix` did to the visuals it was pointed at.
///
/// Unchanged visuals are counted rather than listed: a sweep over an installation checks thousands of files to change a
/// few dozen, and a report naming every one would bury the ones that matter.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixReport {
  pub checked: usize,
  /// Total bytes discarded across every normalized visual.
  pub discarded_size: usize,
  pub failed: usize,
  pub files: Vec<OgfFixFileReport>,
  pub findings: Vec<OgfFixFindingReport>,
  pub is_dry_run: bool,
  pub normalized: usize,
  pub unchanged: usize,
}

impl OgfFixReport {
  pub fn new(outcomes: Vec<OgfFixOutcome>, is_dry_run: bool) -> Self {
    let mut report: Self = Self {
      checked: outcomes.len(),
      discarded_size: 0,
      failed: 0,
      files: Vec::new(),
      findings: Vec::new(),
      is_dry_run,
      normalized: 0,
      unchanged: 0,
    };

    for outcome in outcomes {
      match outcome {
        OgfFixOutcome::Unchanged => report.unchanged += 1,
        OgfFixOutcome::Normalized(file) => {
          report.normalized += 1;
          report.discarded_size += file.discarded_size;
          report.files.push(file);
        }
        OgfFixOutcome::Failed(finding) => {
          report.failed += 1;
          report.findings.push(finding);
        }
      }
    }

    report
  }
}
