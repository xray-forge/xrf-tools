/// The eigen decomposition of a symmetric 3x3 matrix by cyclic Jacobi rotations, as the editor's geometry fits use it
/// (`_matrix33::Meigen`, `Mgc::OrthogonalPlaneFit`), in double precision.
pub(crate) struct LevelBuildEigen;

impl LevelBuildEigen {
  /// Rotation sweeps enough for a 3x3 symmetric matrix to settle in double precision.
  const SWEEPS: usize = 32;

  /// The eigenvalues of `matrix` and its eigenvectors as the columns of the second, `vectors[row][column]`, each column
  /// at the place of the diagonal entry it settled into, as Jacobi leaves them: unsorted.
  pub(crate) fn decompose(mut matrix: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
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

    ([matrix[0][0], matrix[1][1], matrix[2][2]], vectors)
  }

  /// Column `column` of `vectors`.
  pub(crate) fn get_column(vectors: &[[f64; 3]; 3], column: usize) -> [f64; 3] {
    [vectors[0][column], vectors[1][column], vectors[2][column]]
  }
}
