use serde::Serialize;
use xrf_dds::{DdsFormatSupport, DdsRenderer};

/// One renderer's answer about a format.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsConvertCompatibilityReport {
  renderer: String,
  /// `supported`, `unsupported`, or `unverified` - the third being a path nobody has read, not a refusal.
  support: String,
}

impl DdsConvertCompatibilityReport {
  pub fn new(renderer: DdsRenderer, support: DdsFormatSupport) -> Self {
    Self {
      renderer: renderer.label().to_owned(),
      support: match support {
        DdsFormatSupport::Supported => String::from("supported"),
        DdsFormatSupport::Unsupported => String::from("unsupported"),
        DdsFormatSupport::Unverified => String::from("unverified"),
      },
    }
  }
}
