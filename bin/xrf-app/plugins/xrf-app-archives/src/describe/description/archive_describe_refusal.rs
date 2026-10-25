use serde::Serialize;

/// Why an entry was not described.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveDescribeRefusal {
  /// Nothing describes this format yet, and the file is not a container whose shape could be shown instead. The
  /// extension is the authored spelling, empty for a name without one.
  NoDescriber { extension: String },
  /// The entry is larger than the policy admits for a read that holds the payload.
  TooLarge { size: u64, maximum: u32 },
}
