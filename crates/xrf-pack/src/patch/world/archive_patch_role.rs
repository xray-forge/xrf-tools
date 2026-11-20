use std::fmt::{Display, Formatter, Result as FormatResult};

/// Which of the two worlds a comparison is talking about, named alike by the mounts, refusals and transcript labels.
///
/// Not [`ArchivePatchSide`](crate::ArchivePatchSide), which is what one world reported about one entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchivePatchRole {
  /// The release being patched: what a player already has installed.
  Base,
  /// The new build: what the patch is made to deliver.
  Target,
}

impl ArchivePatchRole {
  /// The word a refusal or a summary line names this world by, lower-case for use mid-sentence.
  pub(crate) const fn as_str(self) -> &'static str {
    match self {
      Self::Base => "base",
      Self::Target => "target",
    }
  }

  /// The word a transcript labels this world's block with.
  pub(crate) const fn as_label(self) -> &'static str {
    match self {
      Self::Base => "Base",
      Self::Target => "Target",
    }
  }
}

impl Display for ArchivePatchRole {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    formatter.write_str(self.as_str())
  }
}
