use glam::{DMat3, DVec3};
use xrf_editable_level::EditableObjectTransform;
use xrf_error::XrfResult;
use xrf_math::{Affine3d, Matrix4x4, Vector3d};

/// A transform taking a library object's space into the level's, `rotation * (scale * point) + translation`: a scene
/// object's transform of one scale on every axis, which is how the Level Editor scales an object by hand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelInstanceFit {
  pub(crate) rotation: DMat3,
  pub(crate) scale: f64,
  pub(crate) translation: DVec3,
}

impl LevelInstanceFit {
  /// How far from orthonormal the polar iteration stops, and the most steps it takes; it converges in a handful from
  /// a near rotation.
  const POLAR_TOLERANCE: f64 = 1e-12;
  const POLAR_STEPS: usize = 32;

  /// How far from one a fitted scale may be and still be taken as none: the rounding of the level's corners leaves
  /// a rigid placement's least squares scale a few parts in a million off.
  const UNSCALED: f64 = 1e-4;

  /// The transform taking the triangle `from` onto `to`, corner by corner: the rotation the frames their first two
  /// edges span give, and where `is_scaled`, the scale their perimeters give, else none; none where either is
  /// degenerate.
  pub(crate) fn of_triangles(from: &[DVec3; 3], to: &[DVec3; 3], is_scaled: bool) -> Option<Self> {
    let rotation: DMat3 = Self::get_frame(to)? * Self::get_frame(from)?.transpose();
    let scale: f64 = if is_scaled {
      Self::get_perimeter(to) / Self::get_perimeter(from)
    } else {
      1.0
    };

    (scale.is_finite() && scale > 0.0).then(|| Self {
      rotation,
      scale,
      translation: to[0] - rotation * (from[0] * scale),
    })
  }

  /// The rotation, scale where `is_scaled`, and translation that best take each pair's first point onto its second in
  /// the least squares sense (Umeyama: the orthonormal polar factor of the pairs' cross-covariance, then the scale it
  /// leaves); none where the pairs span no volume or the best orthonormal fit mirrors.
  pub(crate) fn of_pairs(pairs: &[(DVec3, DVec3)], is_scaled: bool) -> Option<Self> {
    if pairs.len() < 3 {
      return None;
    }

    let count: f64 = pairs.len() as f64;
    let from_centre: DVec3 = pairs.iter().map(|(from, _)| *from).sum::<DVec3>() / count;
    let to_centre: DVec3 = pairs.iter().map(|(_, to)| *to).sum::<DVec3>() / count;
    let covariance: DMat3 = pairs.iter().fold(DMat3::ZERO, |sum, (from, to)| {
      sum + Self::outer(*to - to_centre, *from - from_centre)
    });

    if covariance.determinant() <= 0.0 {
      return None;
    }

    let rotation: DMat3 = Self::get_polar_rotation(covariance)?;
    let scale: f64 = if is_scaled {
      let spread: f64 = pairs
        .iter()
        .map(|(from, _)| (*from - from_centre).length_squared())
        .sum();
      let along: f64 = pairs
        .iter()
        .map(|(from, to)| (*to - to_centre).dot(rotation * (*from - from_centre)))
        .sum();

      along / spread
    } else {
      1.0
    };

    if !scale.is_finite() || scale <= 0.0 {
      return None;
    }

    let scale: f64 = if (scale - 1.0).abs() <= Self::UNSCALED {
      1.0
    } else {
      scale
    };

    Some(Self {
      rotation,
      scale,
      translation: to_centre - rotation * (from_centre * scale),
    })
  }

  pub(crate) fn apply(&self, point: DVec3) -> DVec3 {
    self.rotation * (point * self.scale) + self.translation
  }

  /// The scene object transform placing the object as this does, rounded as the editor saves it.
  ///
  /// # Errors
  ///
  /// Returns an error as [`EditableObjectTransform::from_matrix_as_saved`] does.
  pub(crate) fn to_transform(self) -> XrfResult<EditableObjectTransform> {
    EditableObjectTransform::from_matrix_as_saved(&self.to_matrix())
  }

  /// The matrix placing a point as this does, in single precision.
  pub(crate) fn to_matrix(self) -> Matrix4x4 {
    let vector = |vector: DVec3| Vector3d::new(vector.x as f32, vector.y as f32, vector.z as f32);

    Matrix4x4::from(&Affine3d {
      x_axis: vector(self.rotation.x_axis * self.scale),
      y_axis: vector(self.rotation.y_axis * self.scale),
      z_axis: vector(self.rotation.z_axis * self.scale),
      translation: vector(self.translation),
    })
  }

  fn get_perimeter(points: &[DVec3; 3]) -> f64 {
    points[0].distance(points[1]) + points[1].distance(points[2]) + points[2].distance(points[0])
  }

  /// An orthonormal frame of a triangle: its first edge, the normal, and their cross product, as columns; none for a
  /// degenerate triangle.
  fn get_frame(points: &[DVec3; 3]) -> Option<DMat3> {
    let first: DVec3 = (points[1] - points[0]).try_normalize()?;
    let normal: DVec3 = first.cross(points[2] - points[0]).try_normalize()?;

    Some(DMat3::from_cols(first, normal, first.cross(normal)))
  }

  fn outer(left: DVec3, right: DVec3) -> DMat3 {
    DMat3::from_cols(left * right.x, left * right.y, left * right.z)
  }

  /// The orthonormal factor of `matrix`'s polar decomposition, by Newton's iteration `(M + M^-T) / 2`.
  fn get_polar_rotation(matrix: DMat3) -> Option<DMat3> {
    let mut rotation: DMat3 = matrix;

    for _ in 0..Self::POLAR_STEPS {
      let inverse: DMat3 = rotation.inverse();

      if !inverse.is_finite() {
        return None;
      }

      let next: DMat3 = (rotation + inverse.transpose()) * 0.5;
      let change: f64 = (next - rotation)
        .to_cols_array()
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);

      rotation = next;

      if change < Self::POLAR_TOLERANCE {
        break;
      }
    }

    rotation.is_finite().then_some(rotation)
  }
}
