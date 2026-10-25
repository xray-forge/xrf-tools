use serde::Serialize;
use xrf_dds::DdsFormatSupport;

/// What one renderer makes of a format.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureFormatSupport {
  Supported,
  Unsupported,
  /// A path nobody has read, not a refusal.
  Unverified,
}

impl From<DdsFormatSupport> for TextureFormatSupport {
  fn from(support: DdsFormatSupport) -> Self {
    match support {
      DdsFormatSupport::Supported => Self::Supported,
      DdsFormatSupport::Unsupported => Self::Unsupported,
      DdsFormatSupport::Unverified => Self::Unverified,
    }
  }
}
