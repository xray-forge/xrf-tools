use crate::pack::archive_pack_name_table::ArchivePackNameTable;

/// Everything one packing run will write, as the walk that produced it left them.
///
/// The answer of [`crate::pack::archive_pack_source_collector::ArchivePackSourceCollector`], which is where discovery
/// lives; this only carries what it decided so the write phase can read it back.
#[derive(Debug)]
pub(crate) struct ArchivePackSource {
  /// Every file and directory the walk selected, one row per engine name.
  pub(crate) names: ArchivePackNameTable,
  /// Files the rules rejected, reported so a surprising omission is visible rather than silent.
  pub(crate) skipped: usize,
}
