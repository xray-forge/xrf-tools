use xrf_audio_scene::AudioListener;
use xrf_math::glam::Vec3;

/// Where a mono source is heard between the two outputs, by how far to the listener's right it stands: equal power, so
/// a source ahead plays at a half's power in each, as a stereo device's default panning places it. A stereo source is
/// not placed.
pub(crate) struct ListenerPan;

impl ListenerPan {
  /// Each output's gain at a centred source, `cos(π/4)`.
  const CENTRE: f32 = std::f32::consts::FRAC_1_SQRT_2;

  /// The left and right gains of a mono source at `position`, from the listener for a 2D one; one at the listener
  /// itself is centred.
  pub(crate) fn of(listener: &AudioListener, position: [f32; 3], is_2d: bool) -> [f32; 2] {
    // A 2D source's position is from the listener as it faces: right along `x`, ahead along `z`.
    let (across, ahead): (f32, f32) = if is_2d {
      (position[0], position[2])
    } else {
      let offset: Vec3 = Vec3::from(position) - Vec3::from(listener.position);
      let forward: Vec3 = Vec3::from(listener.forward);

      // In the game's left-handed space, up crossed with ahead points right.
      (offset.dot(Vec3::from(listener.up).cross(forward)), offset.dot(forward))
    };

    Self::place(across, across.hypot(ahead))
  }

  /// The gains of a source `across` metres right of `level` metres away on the listener's plane.
  fn place(across: f32, level: f32) -> [f32; 2] {
    if level <= f32::EPSILON {
      return [Self::CENTRE, Self::CENTRE];
    }

    let side: f32 = (across / level).clamp(-1.0, 1.0);
    let angle: f32 = (side + 1.0) * std::f32::consts::FRAC_PI_4;

    [angle.cos(), angle.sin()]
  }
}
