/// What a post-process effect does to the screen at one time, `SPPInfo` as `CPostprocessAnimator::Process` fills it at
/// full strength. An effector's factor then lerps it from [`Self::IDENTITY`], which is how an effect fades in and out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PpeScreen {
  /// `color_base`, as red, green and blue.
  pub base_color: [f32; 3],
  /// `color_add`, as red, green and blue.
  pub add_color: [f32; 3],
  /// `color_gray`, as red, green and blue.
  pub gray_color: [f32; 3],
  /// `gray`.
  pub gray_value: f32,
  /// `blur`.
  pub blur: f32,
  /// `duality.h`.
  pub duality_horizontal: f32,
  /// `duality.v`.
  pub duality_vertical: f32,
  /// `noise.intensity`.
  pub noise_intensity: f32,
  /// `noise.grain`, which the engine refuses at zero or below (`R_ASSERT3` in `CPostprocessAnimator::Process`).
  pub noise_granularity: f32,
  /// `noise.fps`, the frames a second the noise changes at.
  pub noise_fps: f32,
  /// `cm_influence`, how much of the colour grading is mixed in.
  pub color_map_influence: f32,
}

impl PpeScreen {
  /// `pp_identity`, the screen no effect changes, as `CCameraManager` sets it up: its noise fps is 30, where `SPPInfo`
  /// itself starts at 10.
  pub const IDENTITY: Self = Self {
    base_color: [0.5, 0.5, 0.5],
    add_color: [0.0, 0.0, 0.0],
    gray_color: [0.333, 0.333, 0.333],
    gray_value: 0.0,
    blur: 0.0,
    duality_horizontal: 0.0,
    duality_vertical: 0.0,
    noise_intensity: 0.0,
    noise_granularity: 1.0,
    noise_fps: 30.0,
    color_map_influence: 0.0,
  };
}
