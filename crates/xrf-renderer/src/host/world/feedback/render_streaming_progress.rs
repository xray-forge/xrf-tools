use std::time::Instant;

use crate::host::level::part::level_parts::LevelParts;

/// How far the world has streamed a level into its scene, which its load report is made from with what the renderer
/// knows of its textures.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderStreamingProgress {
  /// When the world began opening the level, which its load is timed from.
  pub started: Option<Instant>,
  /// Sectors posted to the scene, of all the level has; the scene may not have taken every one in yet.
  pub sectors: u32,
  pub sectors_total: u32,
  /// The parts finished: every sector posted or failed, the spawn streamed, and each part read whole, whatever the
  /// read answered.
  pub done: LevelParts,
  /// Bytes of every sector's pack posted.
  pub bytes: u64,
}
