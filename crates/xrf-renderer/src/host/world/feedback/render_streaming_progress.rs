use std::time::Instant;

use crate::host::level::part::level_parts::LevelParts;

/// How far the world has streamed a level into its scene, which its load report is made from with what the renderer
/// knows of its textures.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderStreamingProgress {
  /// When the world began opening the level, which its load is timed from.
  pub started: Option<Instant>,
  /// Sectors taken in, of all the level has.
  pub sectors: u32,
  pub sectors_total: u32,
  /// The parts finished: every sector taken in or failed, the spawn streamed, and each part read whole, whatever the
  /// read answered.
  pub done: LevelParts,
  /// Bytes of every sector's pack taken in.
  pub bytes: u64,
}
