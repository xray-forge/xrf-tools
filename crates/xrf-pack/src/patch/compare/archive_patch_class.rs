use serde::Serialize;

/// What a comparison decided about one engine identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchivePatchClass {
  /// Only the target has it, so the patch carries it.
  Added,
  /// Both have it and their payloads differ, so the patch carries the target's.
  Modified,
  /// Only the base has it. The patch cannot carry this, and says so.
  Removed,
}
