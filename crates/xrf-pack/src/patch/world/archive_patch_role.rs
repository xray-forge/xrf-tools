use std::fmt::{Display, Formatter, Result as FormatResult};

/// Which of the two worlds a comparison is talking about.
///
/// A value rather than the `&'static str` this replaced. The words "base" and "target" were spelled at four
/// unconnected sites — both mounts, every refusal, and the transcript's labels — so a rename meant finding all four
/// and a typo at any one of them produced a diagnostic naming a side that does not exist. Two variants also let a
/// guard walk both worlds by role instead of zipping two parallel arrays.
///
/// Not to be confused with [`ArchivePatchSide`](crate::ArchivePatchSide), which is what one world reported about one
/// entry. This is which world that was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ArchivePatchRole {
  /// The release being patched: what a player already has installed.
  Base,
  /// The new build: what the patch is made to deliver.
  Target,
}

impl ArchivePatchRole {
  /// Both roles in the order a diagnostic should walk them, so no caller invents its own.
  pub(crate) const BOTH: [Self; 2] = [Self::Base, Self::Target];

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
