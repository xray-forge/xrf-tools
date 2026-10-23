use serde::Serialize;

/// What became of a session's value, the root every part derives from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum SessionRootChange {
  /// Nothing it was read from moved: the next snapshot shares it.
  Kept,
  /// Something it was read from moved, and its plugin derived it again.
  Derived,
  /// Something it was read from moved, and its plugin derives it again off the commit: the session shows it as it was
  /// until a later event reports it `Derived` or `Failed`.
  Deriving,
  /// Something it was read from moved, and nothing derives it again: the session shows it as it was.
  Stale,
  /// Deriving it again failed: the session stays at the snapshot it had.
  Failed { reason: String },
}
