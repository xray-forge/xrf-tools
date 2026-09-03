use std::path::PathBuf;

/// One file selected for packing: the name it goes in under, and where its bytes are now.
///
/// Produced by the source walk and consumed by the volume writer, so it outlives both and belongs to neither.
#[derive(Clone, Debug)]
pub(crate) struct ArchivePackEntry {
  /// Name as authored: relative to the source root, with X-Ray separators. The engine folds its case on registration.
  pub(crate) name: String,
  pub(crate) path: PathBuf,
}
