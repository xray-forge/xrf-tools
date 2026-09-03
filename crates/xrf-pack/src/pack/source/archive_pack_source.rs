use crate::pack::source::{ArchivePackNameTable, ArchivePackOmissions};

/// Everything one packing run will write, as the walk that produced it left them.
///
/// The answer of [`crate::pack::source::ArchivePackSourceCollector`], which is where discovery lives; this only
/// carries what it decided so the write phase can read it back.
#[derive(Debug)]
pub(crate) struct ArchivePackSource {
  /// Every file and directory the walk selected, one row per engine name.
  pub(crate) names: ArchivePackNameTable,
  /// What the rules left out, counted always and named for a run that is going to say so.
  pub(crate) omitted: ArchivePackOmissions,
}
