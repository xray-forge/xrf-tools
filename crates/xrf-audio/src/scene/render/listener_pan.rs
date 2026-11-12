use xrf_audio_scene::AudioListener;

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
      let offset: [f32; 3] = std::array::from_fn(|axis| position[axis] - listener.position[axis]);

      (
        Self::dot(offset, Self::cross(listener.up, listener.forward)),
        Self::dot(offset, listener.forward),
      )
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
