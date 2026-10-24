use serde::{Deserialize, Serialize};
use xrf_dds::DdsEncodeCandidate;
use xrf_thm::ThmFormat;

/// A format a texture can be written in, of the five worth offering.
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

  /// The `ETFormat` a descriptor names a texture written in this format by, `has_alpha` telling `tfDXT1` from
  /// `tfADXT1`: one encoder, told apart only by whether the alpha means anything. None for BC7, which `ETFormat` has no
  /// member for.
  pub const fn to_thm_format(self, has_alpha: bool) -> Option<ThmFormat> {
    match self {
      Self::Bc1 if has_alpha => Some(ThmFormat::Dxt1Alpha),
      Self::Bc1 => Some(ThmFormat::Dxt1),
      Self::Bc2 => Some(ThmFormat::Dxt3),
      Self::Bc3 => Some(ThmFormat::Dxt5),
      Self::Rgba8 => Some(ThmFormat::Rgba),
      Self::Bc7 => None,
    }
  }
}
