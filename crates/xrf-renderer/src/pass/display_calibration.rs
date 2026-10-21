/// How a display transform is calibrated so that over the game's scenes it shows them as the engine does: each maps
/// middle grey, chroma and the dark its own way, its toe crushing a night or keeping it, so each takes its own stops and
/// saturation before it, its own share of the exposure's adaptation, and its own showing of the game's pictures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisplayCalibration {
  /// The stops the light is lifted by before the transform.
  pub stops: f32,
  /// The saturation the light is graded at before the transform, the grading's own times it.
  pub saturation: f32,
  /// The share of a scene's distance from a daylit exterior the exposure keeps
  /// (`ExposureHistogramUniform::kept_share`).
  pub kept_share: f32,
  /// The stops the game's pictures (its sky, clouds and fog) are shown at against a daylit exterior's exposure, and the
  /// share of the frame's adaptation away from it they follow (`ExposureHistogramUniform::get_picture_share`): over one
  /// where the transform's toe shows a dark sky darker than it shows the land.
  pub picture_stops: f32,
  pub picture_adaptation: f32,
}

impl DisplayCalibration {
  /// The engine's own showing, which grades nothing and holds nothing back.
  pub const NONE: Self = Self {
    stops: 0.0,
    saturation: 1.0,
    kept_share: 1.0,
    picture_stops: 0.0,
    picture_adaptation: 1.0,
  };
}
