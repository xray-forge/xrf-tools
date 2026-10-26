use crate::pack::source::ArchivePackEntry;
use crate::pack::volume::ArchiveAlias;

/// One recorded payload and the entry it was written for, so a match can be proven rather than assumed and its source
/// named by the logical entry rather than by a host path.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ArchiveAliasCandidate<'e> {
  pub(crate) source: &'e ArchivePackEntry,
  pub(crate) alias: ArchiveAlias,
}
