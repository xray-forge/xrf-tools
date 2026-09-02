use serde::{Deserialize, Serialize};

/// Why a binary exists, which is the difference a downloaded artifact cannot show on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum BuildKind {
  /// Built on a developer machine.
  Local,
  /// Continuous integration artifact, built for turnaround rather than size.
  Development,
  /// Release build, carrying the full optimisation the release profile describes.
  Optimized,
}

impl BuildKind {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Local => "local",
      Self::Development => "development",
      Self::Optimized => "optimized",
    }
  }

  /// Read a kind from the value a build recorded, treating anything unrecognised as local.
  pub const fn from_recorded(value: Option<&str>) -> Self {
    match value {
      Some(recorded) if matches!(recorded.as_bytes(), b"development") => Self::Development,
      Some(recorded) if matches!(recorded.as_bytes(), b"optimized") => Self::Optimized,
      _ => Self::Local,
    }
  }
}
