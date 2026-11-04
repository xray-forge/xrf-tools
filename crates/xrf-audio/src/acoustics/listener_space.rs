use crate::scene::audio_listener::AudioListener;

/// Where a placed sound is from the listener, and how its motion and the listener's shift its pitch.
pub(crate) struct ListenerSpace;

impl ListenerSpace {
  /// Metres a second sound travels in air.
  const SPEED_OF_SOUND: f32 = 343.0;

  /// The widest a pitch moves either way, so a camera jumping across a level does not sweep sounds out of hearing.
  const DOPPLER_LIMIT: f32 = 1.5;

  /// The unit vector from `listener` to `position`, in the listener's own space as Steam Audio takes it: `x` right,
  /// `y` up, `-z` ahead. A sound at the listener is ahead.
  pub(crate) fn get_direction(listener: &AudioListener, position: [f32; 3]) -> [f32; 3] {
    let offset: [f32; 3] = std::array::from_fn(|axis| position[axis] - listener.position[axis]);
    let right: [f32; 3] = Self::cross(listener.up, listener.forward);
    let local: [f32; 3] = [
      Self::dot(offset, right),
      Self::dot(offset, listener.up),
      -Self::dot(offset, listener.forward),
    ];
    let length: f32 = Self::dot(local, local).sqrt();

    if length > f32::EPSILON && length.is_finite() {
      local.map(|it| it / length)
    } else {
      [0.0, 0.0, -1.0]
    }
  }

  /// The pitch a sound at `position` moving at `velocity` is heard at, as a factor of its own, the listener moving at
  /// its own velocity: higher while they close in, lower while they part.
  pub(crate) fn get_doppler(listener: &AudioListener, position: [f32; 3], velocity: [f32; 3]) -> f32 {
    let offset: [f32; 3] = std::array::from_fn(|axis| position[axis] - listener.position[axis]);
    let length: f32 = Self::dot(offset, offset).sqrt();

    if length <= f32::EPSILON || !length.is_finite() {
      return 1.0;
    }

    let towards: [f32; 3] = offset.map(|it| it / length);
    // Each speed along the line between them, held under the speed of sound.
    let limit: f32 = Self::SPEED_OF_SOUND * 0.5;
    let listener_speed: f32 = Self::dot(listener.velocity, towards).clamp(-limit, limit);
    let source_speed: f32 = Self::dot(velocity, towards).clamp(-limit, limit);
    let factor: f32 = (Self::SPEED_OF_SOUND + listener_speed) / (Self::SPEED_OF_SOUND + source_speed);

    if factor.is_finite() {
      factor.clamp(1.0 / Self::DOPPLER_LIMIT, Self::DOPPLER_LIMIT)
    } else {
      1.0
    }
  }

  /// In the game's left-handed space, up crossed with ahead points right.
  fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
      a[1] * b[2] - a[2] * b[1],
      a[2] * b[0] - a[0] * b[2],
      a[0] * b[1] - a[1] * b[0],
    ]
  }

  fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
  }
}
