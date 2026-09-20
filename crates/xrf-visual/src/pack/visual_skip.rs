use crate::data::visual_skip_cause::VisualSkipCause;

/// A submesh that produced no geometry, as the packer's internal early return.
///
/// Becomes [`VisualSubmeshContent::Skipped`] verbatim, so every reason below is one a consumer reads.
pub(crate) struct VisualSkip {
  pub(crate) cause: VisualSkipCause,
  pub(crate) reason: String,
}

impl VisualSkip {
  /// Geometry the packer cannot read, which is a gap in coverage rather than a broken file.
  pub(crate) fn unsupported(reason: impl Into<String>) -> Self {
    Self {
      cause: VisualSkipCause::Unsupported,
      reason: reason.into(),
    }
  }

  /// Geometry that contradicts itself, which no amount of added coverage would fix.
  pub(crate) fn malformed(reason: impl Into<String>) -> Self {
    Self {
      cause: VisualSkipCause::Malformed,
      reason: reason.into(),
    }
  }
}
