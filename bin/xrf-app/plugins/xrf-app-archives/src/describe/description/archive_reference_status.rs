use serde::Serialize;

/// Whether the subject being browsed holds what a description named.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveReferenceStatus {
  Present,
  Absent,
  Unknown,
}
