use serde::Serialize;
use xrf_dds::{DdsEncodeCandidate, DdsFormatSupport, DdsRenderer};

/// One renderer's answer about a format.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DdsCompatibilityReport {
  renderer: String,
  /// `supported`, `unsupported`, or `unverified` - the third being a path nobody has read, not a refusal.
  support: String,
}

impl DdsCompatibilityReport {
  pub(crate) fn new(renderer: DdsRenderer, support: DdsFormatSupport) -> Self {
    Self {
      renderer: renderer.label().to_owned(),
      support: match support {
        DdsFormatSupport::Supported => String::from("supported"),
        DdsFormatSupport::Unsupported => String::from("unsupported"),
        DdsFormatSupport::Unverified => String::from("unverified"),
      },
    }
  }

  /// Every renderer's answer about `candidate`, in [`DdsRenderer::ALL`] order.
  pub(crate) fn list_all(candidate: DdsEncodeCandidate) -> Vec<Self> {
    DdsRenderer::ALL
      .into_iter()
      .map(|renderer| Self::new(renderer, candidate.support(renderer)))
      .collect()
  }
}
