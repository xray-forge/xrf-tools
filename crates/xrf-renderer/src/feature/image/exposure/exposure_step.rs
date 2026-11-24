/// How far one frame moves the exposure: the seconds since the last, the share of the way the engine's adaptation
/// moves, whether it goes straight to its target, and whether a flash holds it where it stands.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ExposureStep {
  /// Seconds since the last frame measured.
  pub delta: f32,
  /// The share of the way the engine's adaptation moves at the authored and anchored levels.
  pub engine_blend: f32,
  /// Whether it goes straight to its target, as on the first frame measured.
  pub is_snapped: bool,
  /// Whether a lightning strike lights the frame, or did a moment ago: the physical levels' law takes no step, so a
  /// flash reads as a flash rather than as a lasting change of light. The engine's own adaptation, at the authored and
  /// anchored levels, steps on as the engine's does.
  pub is_held: bool,
}
