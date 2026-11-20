use crate::pack::source::ArchivePackSkipReason;

/// One file the rules left out, by full engine name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchivePackSkippedFile {
  pub name: String,
  pub reason: ArchivePackSkipReason,
}
