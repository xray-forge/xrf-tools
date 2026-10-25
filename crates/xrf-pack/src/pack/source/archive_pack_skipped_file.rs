use crate::pack::ArchivePackSkipReason;

/// One file the rules left out, by full engine name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArchivePackSkippedFile {
  pub(crate) name: String,
  pub(crate) reason: ArchivePackSkipReason,
}
