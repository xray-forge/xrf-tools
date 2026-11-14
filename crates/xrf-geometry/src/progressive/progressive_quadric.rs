/// Garland and Heckbert's quadric error over a point's position and texture coordinate, five dimensions, as the
/// progressive mesh builder of xrLC's tools measures a collapse with (`MxQuadric`, `xrQSlim/src/MxQMetric.cpp`).
///
/// A triangle's quadric is the squared distance from its plane in the five dimensions, unweighted by its area.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ProgressiveQuadric {
  tensor: [[f64; Self::DIMENSIONS]; Self::DIMENSIONS],
  vector: [f64; Self::DIMENSIONS],
  offset: f64,
}

impl ProgressiveQuadric {
  /// A position's three and a texture coordinate's two.
  pub(crate) const DIMENSIONS: usize = 5;

  /// The quadric of no triangle, which a sum starts from.
  pub(crate) const ZERO: Self = Self {
    tensor: [[0.0; Self::DIMENSIONS]; Self::DIMENSIONS],
    vector: [0.0; Self::DIMENSIONS],
    offset: 0.0,
  };

  /// The quadric of the triangle through three points: the identity less the plane's two unit directions' outer
  /// products, so it measures the squared distance off the plane.
  pub(crate) fn of_triangle(
    first: &[f64; Self::DIMENSIONS],
    second: &[f64; Self::DIMENSIONS],
    third: &[f64; Self::DIMENSIONS],
  ) -> Self {
    let along: [f64; Self::DIMENSIONS] = Self::unitize(Self::sub(second, first));
    let mut across: [f64; Self::DIMENSIONS] = Self::sub(third, first);
    let projection: f64 = Self::dot(&along, &across);

    for axis in 0..Self::DIMENSIONS {
      across[axis] -= along[axis] * projection;
    }

    let across: [f64; Self::DIMENSIONS] = Self::unitize(across);
    let first_along: f64 = Self::dot(first, &along);
    let first_across: f64 = Self::dot(first, &across);
    let mut quadric: Self = Self::ZERO;

    for row in 0..Self::DIMENSIONS {
      for column in 0..Self::DIMENSIONS {
        quadric.tensor[row][column] =
          f64::from(u8::from(row == column)) - along[row] * along[column] - across[row] * across[column];
      }

      quadric.vector[row] = along[row] * first_along + across[row] * first_across - first[row];
    }

    quadric.offset = Self::dot(first, first) - first_along * first_along - first_across * first_across;
    quadric
  }

  /// This quadric and `other` summed.
  pub(crate) fn add(&mut self, other: &Self) {
    for row in 0..Self::DIMENSIONS {
      for column in 0..Self::DIMENSIONS {
        self.tensor[row][column] += other.tensor[row][column];
      }

      self.vector[row] += other.vector[row];
    }

    self.offset += other.offset;
  }

  /// The error of a point at `point`: `pᵀ A p + 2 b·p + c`.
  pub(crate) fn evaluate(&self, point: &[f64; Self::DIMENSIONS]) -> f64 {
    let mut product: f64 = 0.0;

    for row in 0..Self::DIMENSIONS {
      product += point[row] * Self::dot(&self.tensor[row], point);
    }

    product + 2.0 * Self::dot(&self.vector, point) + self.offset
  }

  fn sub(first: &[f64; Self::DIMENSIONS], second: &[f64; Self::DIMENSIONS]) -> [f64; Self::DIMENSIONS] {
    std::array::from_fn(|axis| first[axis] - second[axis])
  }

  fn dot(first: &[f64; Self::DIMENSIONS], second: &[f64; Self::DIMENSIONS]) -> f64 {
    first.iter().zip(second).map(|(first, second)| first * second).sum()
  }

  /// `vector` scaled to unit length, left as it is when of no length (`unitize`, `MxVector.h`).
  fn unitize(vector: [f64; Self::DIMENSIONS]) -> [f64; Self::DIMENSIONS] {
    let length_squared: f64 = Self::dot(&vector, &vector);

    if length_squared == 0.0 || length_squared == 1.0 {
      return vector;
    }

    let length: f64 = length_squared.sqrt();

    vector.map(|value| value / length)
  }
}
