use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use xrf_dds::{DdsEncodeAttempt, DdsFormatSupport, DdsRenderer};

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

/// One renderer's answer about a format.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsConvertCompatibilityReport {
  renderer: String,
  /// `supported`, `unsupported`, or `unverified` - the third being a path nobody has read, not a refusal.
  support: String,
}

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
        .map(|renderer| DdsConvertCompatibilityReport {
          renderer: renderer.label().to_owned(),
          support: match attempt.candidate.support(renderer) {
            DdsFormatSupport::Supported => String::from("supported"),
            DdsFormatSupport::Unsupported => String::from("unsupported"),
            DdsFormatSupport::Unverified => String::from("unverified"),
          },
        })
        .collect(),
    }
  }
}
