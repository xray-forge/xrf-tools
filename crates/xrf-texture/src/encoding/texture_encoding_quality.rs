use serde::{Deserialize, Serialize};
use xrf_dds::Quality;

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

impl TextureEncodingQuality {
  pub const fn to_quality(self) -> Quality {
    match self {
      Self::Fast => Quality::Fast,
      Self::Normal => Quality::Normal,
      Self::Slow => Quality::Slow,
    }
  }
}
