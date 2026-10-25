use serde::Serialize;

/// What a level's counts are counting.
///
/// Carried rather than inferred because entry counts mislead where entry sizes do not agree: an archive whose last two
/// hundred entries are level meshes sits at ninety-nine per cent for a third of its run. A reader that cannot tell a
/// count of things from a count of bytes has to hardcode per-operation knowledge to render either one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressUnit {
  /// Discrete things: files, entries, checks, volumes.
  #[default]
  Items,
  /// Bytes, rendered through the reader's own size formatting.
  Bytes,
}
