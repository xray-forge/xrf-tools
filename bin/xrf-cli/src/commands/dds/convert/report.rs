use std::path::Path;

use serde::Serialize;
use xrf_dds::DdsEncodeAttempt;

use crate::commands::dds::convert::dds_convert_candidate_report::DdsConvertCandidateReport;

/// What `dds convert` wrote, and what the format it wrote in cost.
///
/// Every figure is relative to the source as decoded. For a texture already stored in a lossy format the source is
/// itself lossy, so the error reported here is what this conversion adds on top - never distance from an original.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsConvertReport {
  source: String,
  destination: String,
  width: u32,
  height: u32,
  mipmap_levels: u32,
  /// The format that was written.
  written: DdsConvertCandidateReport,
  /// Every other candidate, weighed against the same levels. Empty unless `--compare` was given.
  candidates: Vec<DdsConvertCandidateReport>,
}

impl DdsConvertReport {
  pub fn new(source: &Path, destination: &Path, written: &DdsEncodeAttempt, compared: &[DdsEncodeAttempt]) -> Self {
    let metadata = written.file.metadata();

    Self {
      source: xrf_utils::to_portable_path_string(source),
      destination: xrf_utils::to_portable_path_string(destination),
      width: metadata.width,
      height: metadata.height,
      mipmap_levels: metadata.mipmap_levels,
      written: DdsConvertCandidateReport::new(written),
      candidates: compared.iter().map(DdsConvertCandidateReport::new).collect(),
    }
  }
}
