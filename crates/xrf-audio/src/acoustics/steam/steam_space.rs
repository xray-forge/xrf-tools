use audionimbus::{CoordinateSystem, Point, Vector3};

use crate::scene::audio_listener::AudioListener;

/// The game's left-handed space (`x` right, `y` up, `z` ahead) in Steam Audio's right-handed one, whose listener looks
/// along `-z`: flipping `z` maps one onto the other.
pub(crate) struct SteamSpace;

impl SteamSpace {
  pub(crate) fn to_point([x, y, z]: [f32; 3]) -> Point {
    Point::new(x, y, -z)
  }

  /// The listener's position and axes.
  pub(crate) fn to_listener(listener: &AudioListener) -> CoordinateSystem {
    let [ux, uy, uz] = listener.up;
    let [fx, fy, fz] = listener.forward;
    // In the game's space, up crossed with ahead points right.
    let right: [f32; 3] = [uy * fz - uz * fy, uz * fx - ux * fz, ux * fy - uy * fx];

    CoordinateSystem {
      right: Vector3::new(right[0], right[1], -right[2]),
      up: Vector3::new(ux, uy, -uz),
      ahead: Vector3::new(fx, fy, -fz),
      origin: Self::to_point(listener.position),
    }
  }

  /// A source at `position`, its axes Steam Audio's own.
  pub(crate) fn to_source(position: [f32; 3]) -> CoordinateSystem {
    CoordinateSystem {
      origin: Self::to_point(position),
      ..CoordinateSystem::default()
    }
  }
}
