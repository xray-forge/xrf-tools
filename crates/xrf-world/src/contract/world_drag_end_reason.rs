use serde::{Deserialize, Serialize};

/// Why a drag over a viewport ended.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WorldDragEndReason {
  /// The button was let go.
  Released,
  /// Something took the drag away: Escape, another window, a lost pointer; an edit drag reverts on it.
  Cancelled,
}
