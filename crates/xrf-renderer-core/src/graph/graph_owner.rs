use serde::Serialize;

/// Whose a pass is, which the timer sums its cost by: the frame's own work, or one part of a frame drawing several.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GraphOwner {
  /// The frame's own work, which every pass is until another owner is named.
  Frame,
  /// One part of a frame drawing several, as a view, by the caller's number for it.
  Part(u32),
}
