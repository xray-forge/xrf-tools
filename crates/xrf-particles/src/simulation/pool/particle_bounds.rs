use glam::Vec3;
use xrf_math::EPS_L;

/// `vis.box` and `vis.sphere`: what an effect's particles cover, grown by their largest size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleBounds {
  pub min: Vec3,
  pub max: Vec3,
}

impl ParticleBounds {
  /// `box.set(p, p).grow(EPS_L)`: a point, as an effect not playing reports where it was placed.
  pub fn around_point(point: Vec3) -> Self {
    Self {
      min: point - Vec3::splat(EPS_L),
      max: point + Vec3::splat(EPS_L),
    }
  }

  /// `Fbox::merge`.
  pub fn merge(&self, other: &Self) -> Self {
    Self {
      min: self.min.min(other.min),
      max: self.max.max(other.max),
    }
  }

  /// `Fbox::getsphere`: the box's centre and the distance to its corner.
  pub fn get_sphere(&self) -> (Vec3, f32) {
    let center: Vec3 = (self.min + self.max) / 2.0;

    (center, self.max.distance(center))
  }
}

#[cfg(test)]
mod tests {
  use glam::Vec3;

  use super::ParticleBounds;

  #[test]
  fn merges_and_measures_a_sphere_around_the_box() {
    let bounds: ParticleBounds = ParticleBounds {
      min: Vec3::ZERO,
      max: Vec3::ONE,
    }
    .merge(&ParticleBounds {
      min: Vec3::new(-1.0, 0.0, 0.0),
      max: Vec3::new(0.0, 1.0, 1.0),
    });

    assert_eq!(bounds.min, Vec3::new(-1.0, 0.0, 0.0));
    assert_eq!(
      bounds.get_sphere(),
      (Vec3::new(0.0, 0.5, 0.5), Vec3::new(1.0, 0.5, 0.5).length())
    );
  }
}
