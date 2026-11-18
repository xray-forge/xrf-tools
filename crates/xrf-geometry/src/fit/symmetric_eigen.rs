use glam::DVec3;

/// The eigen decomposition of a symmetric 3x3 matrix by cyclic Jacobi rotations, in double precision: the solver the
/// SDK's fits take their axes from (`_matrix33::Meigen`, `Mgc::OrthogonalPlaneFit`, `ComputeOBB_RAPID`).
///
/// A pair is skipped once its entry is within `f64::EPSILON` of the two diagonal entries it couples, a stop that does
/// not depend on the points' scale, so a level's coordinates and a bone's settle alike.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SymmetricEigen {
  /// The eigenvalues, each at the place of the diagonal entry it settled into: unsorted, as Jacobi leaves them.
  pub values: [f64; 3],
  /// The unit eigenvector of each value, at the same place.
  pub vectors: [DVec3; 3],
}

impl SymmetricEigen {
  /// Rotation sweeps enough for a 3x3 symmetric matrix to settle in double precision.
  const SWEEPS: usize = 32;

  /// The decomposition of `matrix`, given by rows; only its symmetric part has a meaning.
  #[must_use]
  pub fn of(mut matrix: [[f64; 3]; 3]) -> Self {
    let mut vectors: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..Self::SWEEPS {
      for (p, q) in [(0, 1), (0, 2), (1, 2)] {
        if matrix[p][q].abs() <= f64::EPSILON * (matrix[p][p].abs() + matrix[q][q].abs()) {
          continue;
        }

        let theta: f64 = (matrix[q][q] - matrix[p][p]) / (2.0 * matrix[p][q]);
        let tangent: f64 = theta.signum() / (theta.abs() + theta.mul_add(theta, 1.0).sqrt());
        let cosine: f64 = 1.0 / tangent.mul_add(tangent, 1.0).sqrt();
        let sine: f64 = tangent * cosine;

        for row in &mut matrix {
          let (a, b) = (row[p], row[q]);

          row[p] = cosine * a - sine * b;
          row[q] = sine * a + cosine * b;
        }

        let (row_p, row_q) = (matrix[p], matrix[q]);

        matrix[p] = [0, 1, 2].map(|column| cosine * row_p[column] - sine * row_q[column]);
        matrix[q] = [0, 1, 2].map(|column| sine * row_p[column] + cosine * row_q[column]);

        for vector in &mut vectors {
          let (a, b) = (vector[p], vector[q]);

          vector[p] = cosine * a - sine * b;
          vector[q] = sine * a + cosine * b;
        }
      }
    }

    Self {
      values: [matrix[0][0], matrix[1][1], matrix[2][2]],
      vectors: [0, 1, 2].map(|column| DVec3::new(vectors[0][column], vectors[1][column], vectors[2][column])),
    }
  }

  /// The place of the least eigenvalue by `f64::total_cmp`; the first of equals.
  #[must_use]
  pub fn get_least(&self) -> usize {
    (0..3)
      .min_by(|a, b| self.values[*a].total_cmp(&self.values[*b]))
      .unwrap_or_default()
  }

  /// The place of the greatest eigenvalue by `f64::total_cmp`; the last of equals.
  #[must_use]
  pub fn get_greatest(&self) -> usize {
    (0..3)
      .max_by(|a, b| self.values[*a].total_cmp(&self.values[*b]))
      .unwrap_or_default()
  }
}
