/// How a placed voice is heard through the acoustics as of the control side's last tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VoiceSpatialParameters {
  /// Unit vector from the listener to the sound, in the listener's own space as Steam Audio takes it: `x` right, `y`
  /// up, `-z` ahead.
  pub(crate) direction: [f32; 3],
  /// The distance curve's gain, from 0 to 1.
  pub(crate) attenuation: f32,
  /// What the air lets through in each band, from 0 to 1.
  pub(crate) air_absorption: [f32; 3],
  /// The share of the sound reaching the listener unobstructed, from 0 (wholly behind walls) to 1.
  pub(crate) occlusion: f32,
  /// What the walls in the way let through in each band, from 0 to 1.
  pub(crate) transmission: [f32; 3],
  /// How much of it the listener's reverb hears, from 0.
  pub(crate) reverb_send: f32,
  /// Whether it reaches the ears through the HRTF rather than panned.
  pub(crate) is_binaural: bool,
}

impl Default for VoiceSpatialParameters {
  /// Ahead, at full volume, unobstructed and dry.
  fn default() -> Self {
    Self {
      direction: [0.0, 0.0, -1.0],
      attenuation: 1.0,
      air_absorption: [1.0; 3],
      occlusion: 1.0,
      transmission: [1.0; 3],
      reverb_send: 0.0,
      is_binaural: true,
    }
  }
}
