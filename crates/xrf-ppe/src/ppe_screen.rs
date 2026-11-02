/// What a post-process effect does to the screen at one time, `SPPInfo` as `CPostprocessAnimator::Process` fills it at
/// full strength. An effector's factor then fades it from [`Self::IDENTITY`] ([`Self::lerp`]), which is how an effect
/// fades in and out.
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

  /// The screen at `factor` of its strength, as `SPPInfo::lerp` fades it from [`Self::IDENTITY`]: the colours, gray,
  /// blur, duality and colour map influence move that far from the identity's, while the noise keeps this screen's, as
  /// the engine copies it rather than fading it. The factor is clamped to `0..=1`.
  pub fn lerp(&self, factor: f32) -> Self {
    let factor: f32 = factor.clamp(0.0, 1.0);
    let identity: Self = Self::IDENTITY;
    let mix = |from: f32, to: f32| from + (to - from) * factor;
    let mix_color = |from: [f32; 3], to: [f32; 3]| [mix(from[0], to[0]), mix(from[1], to[1]), mix(from[2], to[2])];

    Self {
      base_color: mix_color(identity.base_color, self.base_color),
      add_color: mix_color(identity.add_color, self.add_color),
      gray_color: mix_color(identity.gray_color, self.gray_color),
      gray_value: mix(identity.gray_value, self.gray_value),
      blur: mix(identity.blur, self.blur),
      duality_horizontal: mix(identity.duality_horizontal, self.duality_horizontal),
      duality_vertical: mix(identity.duality_vertical, self.duality_vertical),
      noise_intensity: self.noise_intensity,
      noise_granularity: self.noise_granularity,
      noise_fps: self.noise_fps,
      color_map_influence: mix(identity.color_map_influence, self.color_map_influence),
    }
  }
}
