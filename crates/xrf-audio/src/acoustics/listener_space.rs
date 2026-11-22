use xrf_audio_scene::AudioListener;
use xrf_math::glam::Vec3;

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
    let offset: Vec3 = Vec3::from(position) - Vec3::from(listener.position);
    let up: Vec3 = Vec3::from(listener.up);
    let forward: Vec3 = Vec3::from(listener.forward);
    // In the game's left-handed space, up crossed with ahead points right.
    let local: Vec3 = Vec3::new(offset.dot(up.cross(forward)), offset.dot(up), -offset.dot(forward));
    let length: f32 = local.dot(local).sqrt();

    if length > f32::EPSILON && length.is_finite() {
      (local / length).to_array()
    } else {
      [0.0, 0.0, -1.0]
    }
  }

  /// The pitch a sound at `position` moving at `velocity` is heard at, as a factor of its own, the listener moving at
  /// its own velocity: higher while they close in, lower while they part.
  pub(crate) fn get_doppler(listener: &AudioListener, position: [f32; 3], velocity: [f32; 3]) -> f32 {
    let offset: Vec3 = Vec3::from(position) - Vec3::from(listener.position);
    let length: f32 = offset.dot(offset).sqrt();

    if length <= f32::EPSILON || !length.is_finite() {
      return 1.0;
    }

    let towards: Vec3 = offset / length;
    // Each speed along the line between them, held under the speed of sound.
    let limit: f32 = Self::SPEED_OF_SOUND * 0.5;
    let listener_speed: f32 = Vec3::from(listener.velocity).dot(towards).clamp(-limit, limit);
    let source_speed: f32 = Vec3::from(velocity).dot(towards).clamp(-limit, limit);
    let factor: f32 = (Self::SPEED_OF_SOUND + listener_speed) / (Self::SPEED_OF_SOUND + source_speed);

    if factor.is_finite() {
      factor.clamp(1.0 / Self::DOPPLER_LIMIT, Self::DOPPLER_LIMIT)
    } else {
      1.0
    }
  }
}
