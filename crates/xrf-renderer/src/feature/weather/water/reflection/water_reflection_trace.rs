/// How the enhanced water's reflection is traced this frame.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WaterReflectionTrace {
  /// How far its misses are traced, none for not.
  pub distance: f32,
  /// How many frames a pixel waits between its rays.
  pub turns: u32,
  /// The frame's count, which picks the pixels that trace.
  pub frame: u32,
  /// How many times its turns a still, dim reflection waits, at least one.
  pub dim_turns: u32,
  /// The steps the reflection marches, at least one.
  pub march_steps: u32,
  /// Whether every pixel traces every frame, the screen answering only plants past the grass's cards
  /// (`RenderReflectionSettings::is_water_fully_traced`).
  pub is_full: bool,
}
