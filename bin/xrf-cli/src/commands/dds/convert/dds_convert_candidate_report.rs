use std::time::Duration;

use serde::Serialize;
use xrf_dds::{DdsEncodeAttempt, DdsRenderer};

use crate::commands::dds::convert::dds_convert_compatibility_report::DdsConvertCompatibilityReport;

/// What one candidate format cost, whether it was written or only weighed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsConvertCandidateReport {
  format: String,
  /// Bytes on disk, header included.
  file_bytes: u64,
  /// Bytes once uploaded, which is the whole mip chain without the header.
  gpu_bytes: u64,
  #[serde(with = "xrf_utils::duration_ms")]
  encode_duration: Duration,
  /// Peak signal-to-noise ratio against the source's own pixels, absent where nothing was lost.
  psnr: Option<f64>,
  /// Root mean square error per channel, in the eight-bit units the pixels are stored in.
  channel_rmse: [f64; 4],
  /// Which renderer path loads a texture in this format.
  compatibility: Vec<DdsConvertCompatibilityReport>,
}

impl DdsConvertCandidateReport {
  pub fn new(attempt: &DdsEncodeAttempt) -> Self {
    Self {
      format: attempt.candidate.label().to_owned(),
      file_bytes: attempt.file_bytes,
      gpu_bytes: attempt.gpu_bytes,
      encode_duration: attempt.encode_duration,
      psnr: attempt.difference.psnr,
      channel_rmse: attempt.difference.channel_rmse,
      compatibility: DdsRenderer::ALL
        .into_iter()
        .map(|renderer| DdsConvertCompatibilityReport::new(renderer, attempt.candidate.support(renderer)))
        .collect(),
    }
  }
}
