use std::time::Duration;

use serde::Serialize;
use xrf_dds::{DdsEncodeAttempt, DdsRenderer};
use xrf_texture::TextureEncodingFormat;

use crate::encoding::TextureRendererSupport;

/// What one candidate cost and what it lost, weighed against the texture as it is now.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEncodingReport {
  pub format: TextureEncodingFormat,
  /// The format's name with the DXT name the descriptor and the SDK use for it.
  pub label: String,
  /// Bytes on disk, header included.
  pub file_bytes: u64,
  /// Bytes once uploaded, which is the whole mip chain without the header.
  pub gpu_bytes: u64,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub encode_duration: Duration,
  /// Peak signal-to-noise ratio, absent where nothing was lost.
  pub psnr: Option<f64>,
  /// Root mean square error per channel, in the eight-bit units the pixels are stored in.
  pub channel_rmse: [f64; 4],
  /// What the renderers together make of the format, as a sentence a badge can show.
  pub support_summary: String,
  pub compatibility: Vec<TextureRendererSupport>,
}

impl TextureEncodingReport {
  /// What one encode attempt says, without its bytes.
  pub fn of(attempt: &DdsEncodeAttempt) -> Self {
    Self {
      format: TextureEncodingFormat::of(attempt.candidate),
      label: attempt.candidate.label().to_owned(),
      file_bytes: attempt.file_bytes,
      gpu_bytes: attempt.gpu_bytes,
      encode_duration: attempt.encode_duration,
      psnr: attempt.difference.psnr,
      channel_rmse: attempt.difference.channel_rmse,
      support_summary: attempt.candidate.support_summary(),
      compatibility: DdsRenderer::ALL
        .into_iter()
        .map(|renderer| TextureRendererSupport {
          renderer: renderer.label().to_owned(),
          support: attempt.candidate.support(renderer).into(),
        })
        .collect(),
    }
  }
}
