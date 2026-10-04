use glam::Vec3;

/// `Fvector` operations whose float steps the simulation must repeat as `xrMiscMath/vector.cpp` takes them.
pub(crate) trait EngineVector {
  /// `normalize_safe`: scaled by the inverse square root of its square length, unless that is denormal.
  fn normalize_safe(self) -> Self;
}

impl EngineVector for Vec3 {
  fn normalize_safe(self) -> Self {
    let length_sqr: f32 = self.length_squared();

    if length_sqr > f32::MIN_POSITIVE {
      self * (1.0 / length_sqr).sqrt()
    } else {
      self
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::Vec3;

  use super::EngineVector;

  #[test]
  fn leaves_a_zero_vector_as_it_is() {
    assert_eq!(Vec3::ZERO.normalize_safe(), Vec3::ZERO);
  }

  #[test]
  fn scales_by_the_inverse_square_root() {
    assert_eq!(
      Vec3::new(3.0, 0.0, 4.0).normalize_safe(),
      Vec3::new(3.0, 0.0, 4.0) * (1.0f32 / 25.0).sqrt()
    );
  }
}
