use std::time::Duration;

use serde::{Deserialize, Serialize};
use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsFile, DdsFormatSupport, DdsRenderer, Quality};
use xrf_job::JobOutcome;
use xrf_vfs::XrayRoots;

use crate::core::session::DocumentSessionId;
use crate::core::types::TauriResult;

use crate::plugins::textures::source::TextureSource;

/// A format the base texture can be written in, of the five worth offering.
///
/// A plugin-side mirror of [`DdsEncodeCandidate`] rather than the crate's own enum, for the reason every wire type
/// here is one: `xrf-dds` is a pure image crate and carries no bindings feature, and a surface naming a format wants
/// a name that cannot change under it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureEncodingFormat {
  Bc1,
  Bc2,
  Bc3,
  Rgba8,
  Bc7,
}

/// How hard the encoder works, which trades seconds for fidelity.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureEncodingQuality {
  Fast,
  Normal,
  /// The default, because everything but BC7 costs pennies at it.
  #[default]
  Slow,
}

/// One renderer's answer about a format.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureRendererSupport {
  pub renderer: String,
  /// `supported`, `unsupported`, or `unverified` - the third being a path nobody has read, not a refusal.
  pub support: String,
}

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

/// The texture as it stands, for the row a comparison is read against.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEncodingCurrent {
  /// Format name from the file's own header, which is not always one of the five candidates.
  pub label: String,
  pub file_bytes: u64,
  pub gpu_bytes: u64,
  pub width: u32,
  pub height: u32,
  pub mipmap_levels: u32,
}

/// Every candidate weighed against one texture, with the texture itself for a baseline.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEncodingComparison {
  pub session_id: DocumentSessionId,
  pub source: TextureSource,
  pub roots: XrayRoots,
  /// Whether every candidate was weighed or the run stopped because it was asked to.
  ///
  /// A cancelled comparison still reports what it managed, and the session still holds those encodes: a candidate it
  /// reached is a real measurement and a real set of bytes, whatever happened after it.
  pub outcome: JobOutcome,
  /// The texture the encodes were made from, which a save has to name to claim them.
  pub reference: String,
  pub current: TextureEncodingCurrent,
  /// The candidates weighed, in the order [`DdsEncodeCandidate::ALL`] lists them, cheapest first.
  pub candidates: Vec<TextureEncodingReport>,
}

/// The encodes one comparison produced, kept until somebody saves one or asks for another texture.
pub struct TextureEncodingSession {
  pub session_id: DocumentSessionId,
  pub roots: XrayRoots,
  /// The texture these were encoded from, so a save cannot write one texture's bytes over another's file.
  ///
  /// The source rather than its label: two files in different trees can share an engine reference, and a label a
  /// standalone file gets from its own stem is not unique at all.
  pub source: TextureSource,
  /// What to call it in a message, carried beside the source because a source is an address rather than a name.
  pub label: String,
  pub attempts: Vec<DdsEncodeAttempt>,
}

impl TextureEncodingSession {
  /// The encoded file for one candidate, or nothing when this session never weighed it.
  pub fn get(&self, format: TextureEncodingFormat) -> Option<&DdsFile> {
    let candidate: DdsEncodeCandidate = format.to_candidate();

    self
      .attempts
      .iter()
      .find(|attempt| attempt.candidate == candidate)
      .map(|attempt| &attempt.file)
  }

  /// Refuse formats this comparison did not finish encoding.
  pub fn require(&self, format: TextureEncodingFormat) -> TauriResult<&DdsFile> {
    self.get(format).ok_or_else(|| {
      format!(
        "The held comparison of '{}' does not carry that format; compare the formats again",
        self.label
      )
    })
  }

  /// What the comparison says, which is everything about the attempts except their bytes.
  pub fn to_comparison(&self, current: TextureEncodingCurrent, outcome: JobOutcome) -> TextureEncodingComparison {
    TextureEncodingComparison {
      session_id: self.session_id,
      source: self.source.clone(),
      roots: self.roots.clone(),
      outcome,
      reference: self.label.clone(),
      current,
      candidates: self.attempts.iter().map(TextureEncodingReport::of).collect(),
    }
  }
}

impl TextureEncodingReport {
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
          support: String::from(match attempt.candidate.support(renderer) {
            DdsFormatSupport::Supported => "supported",
            DdsFormatSupport::Unsupported => "unsupported",
            DdsFormatSupport::Unverified => "unverified",
          }),
        })
        .collect(),
    }
  }
}

impl TextureEncodingFormat {
  pub const fn of(candidate: DdsEncodeCandidate) -> Self {
    match candidate {
      DdsEncodeCandidate::Bc1 => Self::Bc1,
      DdsEncodeCandidate::Bc2 => Self::Bc2,
      DdsEncodeCandidate::Bc3 => Self::Bc3,
      DdsEncodeCandidate::Rgba8 => Self::Rgba8,
      DdsEncodeCandidate::Bc7 => Self::Bc7,
    }
  }

  pub const fn to_candidate(self) -> DdsEncodeCandidate {
    match self {
      Self::Bc1 => DdsEncodeCandidate::Bc1,
      Self::Bc2 => DdsEncodeCandidate::Bc2,
      Self::Bc3 => DdsEncodeCandidate::Bc3,
      Self::Rgba8 => DdsEncodeCandidate::Rgba8,
      Self::Bc7 => DdsEncodeCandidate::Bc7,
    }
  }
}

impl TextureEncodingQuality {
  pub const fn to_quality(self) -> Quality {
    match self {
      Self::Fast => Quality::Fast,
      Self::Normal => Quality::Normal,
      Self::Slow => Quality::Slow,
    }
  }
}
