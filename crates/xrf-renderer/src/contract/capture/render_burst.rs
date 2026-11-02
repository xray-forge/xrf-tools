use serde::{Deserialize, Serialize};

/// A run of consecutive frames asked of a viewport, drawn on a clock of its own a fixed step apart from a set time, so
/// two bursts from the same time draw the same scene however long their frames take; each read back whole as a
/// screenshot is, with its G-buffer's marks.
///
/// Its first frame restarts what the view carries from frame to frame (the wind's drift, the temporal resolve's
/// history and jitter), so what it draws depends on its own frames alone. A reference burst draws each frame's time
/// `samples` times over, its clock held, at as many places of the resolve's jitter, unresolved and at the viewport's
/// own size: what a perfect temporal resolve converges to at that time, once averaged.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderBurst {
  /// The scene's seconds, since its level began opening, its first frame is drawn at.
  pub at: f32,
  /// Seconds between its frames.
  pub step: f32,
  /// Frames drawn before the first read back, for what builds over frames to settle from its restart.
  pub lead_in: u32,
  /// Frames read back, one after another.
  pub frames: u32,
  /// Samples each frame read back is drawn at for a reference; none for the frames as the view resolves them.
  pub samples: Option<u32>,
}
