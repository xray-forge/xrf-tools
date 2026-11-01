/// What a frame's reflections are traced over besides their settings: the frame's count, what the last frame left them,
/// whether the device traces rays, and whether the level is wet.
#[derive(Clone, Copy, Debug, Default)]
pub struct ReflectionTraceFrame {
  pub frame: u32,
  /// Whether the last frame's reflections are there to carry.
  pub has_history: bool,
  /// Whether the last frame's scene is there to read.
  pub has_source: bool,
  /// Whether the device traces rays.
  pub can_trace: bool,
  pub is_wet: bool,
}
