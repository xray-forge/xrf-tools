use serde::Serialize;

/// What a description's reference lookups searched.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveDescribeScope {
  /// The merged name table of the volumes the explorer has open.
  Volumes { volumes: usize },
  /// Every source a mounted world searches, which is what the engine would search.
  World,
}
