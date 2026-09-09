use std::fmt::{Display, Formatter, Result as FormatResult};

use serde::{Deserialize, Serialize};

/// What the two sides of a comparison are to each other, which is the only thing that says what a base-only entry
/// means.
///
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchivePatchShape {
  /// The target adds to and overrides the base, and says nothing about the rest of it.
  ///
  /// What a mod is: a gamedata tree, or the loose half of an installation, laid over a release. Base-only entries are
  /// untouched files rather than deletions, so they are not classified at all — the run answers "what does this
  /// overlay actually change", which is also what makes the published patch smaller than the folder it came from.
  #[default]
  Overlay,
  /// Both sides are complete releases, so what the base holds and the target lacks was dropped.
  ///
  /// The shape a version-to-version patch is built in: `v1.0` against `v1.1`. Base-only entries are reported, and
  /// under `is_strict` they fail the run, because the format cannot carry them and shipping the patch would silently
  /// leave them behind.
  Release,
}

impl ArchivePatchShape {
  /// Whether entries only the base holds are a finding rather than the ordinary case.
  pub const fn is_reporting_removals(self) -> bool {
    matches!(self, Self::Release)
  }
}

impl Display for ArchivePatchShape {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    formatter.write_str(match self {
      Self::Overlay => "overlay",
      Self::Release => "release",
    })
  }
}
