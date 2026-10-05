use std::time::Instant;

/// How far the world has streamed a level into its scene, which its load report is made from with what the renderer
/// knows of its textures, grass and particles.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderStreamingProgress {
  /// When the world began opening the level, which its load is timed from.
  pub started: Option<Instant>,
  /// Sectors taken in, of all the level has.
  pub sectors: u32,
  pub sectors_total: u32,
  /// Whether every sector was taken in or failed.
  pub is_sectors_done: bool,
  pub is_spawn_done: bool,
  pub is_lights_done: bool,
  /// Bytes of every sector's pack taken in.
  pub bytes: u64,
}
