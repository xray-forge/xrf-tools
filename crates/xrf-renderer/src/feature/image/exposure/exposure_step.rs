/// How far one frame moves the exposure: the seconds since the last, the share of the way the engine's adaptation
/// moves, and whether it goes straight to its target.
#[derive(Clone, Copy, Debug, Default)]
pub struct ExposureStep {
  /// Seconds since the last frame measured.
  pub delta: f32,
  /// The share of the way the engine's adaptation moves at the authored and anchored levels.
  pub engine_blend: f32,
  /// Whether it goes straight to its target, as on the first frame measured.
  pub is_snapped: bool,
}
