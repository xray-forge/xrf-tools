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
  /// The spelling `XRF_BUILD_KIND` records and serialization writes.
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Local => "local",
      Self::Development => "development",
      Self::Optimized => "optimized",
    }
  }

  /// Read a kind from the value a build recorded, treating anything unrecognised as local.
  pub const fn from_recorded(value: Option<&str>) -> Self {
    let Some(recorded) = value else {
      return Self::Local;
    };

    // Bytes, because a `const fn` cannot compare strings.
    match recorded.as_bytes() {
      b"development" => Self::Development,
      b"optimized" => Self::Optimized,
      _ => Self::Local,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::BuildKind;

  #[test]
  fn a_recorded_kind_reads_back_as_its_spelling() {
    for kind in [BuildKind::Development, BuildKind::Optimized] {
      assert_eq!(BuildKind::from_recorded(Some(kind.as_str())), kind);
    }
  }

  #[test]
  fn anything_else_recorded_is_local() {
    assert_eq!(BuildKind::from_recorded(None), BuildKind::Local);
    assert_eq!(BuildKind::from_recorded(Some("")), BuildKind::Local);
    assert_eq!(
      BuildKind::from_recorded(Some("Optimized")),
      BuildKind::Local,
      "the spelling is case-sensitive"
    );
  }
}
