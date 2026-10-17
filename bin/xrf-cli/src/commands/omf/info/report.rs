use serde::Serialize;
use xrf_omf::OmfFile;

use crate::commands::omf::info::omf_motion_report::OmfMotionReport;
use crate::commands::omf::info::omf_part_report::OmfPartReport;

/// What `omf info` read out of a motion container.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmfInfoReport {
  bones: usize,
  motions: Vec<OmfMotionReport>,
  parts: Vec<OmfPartReport>,
  version: u16,
}

impl OmfInfoReport {
  pub fn new(file: &OmfFile) -> Self {
    Self {
      bones: file.parameters.get_bones_count(),
      motions: OmfMotionReport::list(file),
      parts: OmfPartReport::list(file),
      version: file.parameters.version,
    }
  }
}
