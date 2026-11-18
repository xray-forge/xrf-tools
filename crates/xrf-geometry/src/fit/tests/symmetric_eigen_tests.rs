use glam::{DMat3, DQuat, DVec3};

use crate::fit::symmetric_eigen::SymmetricEigen;

#[test]
fn finds_the_axes_of_a_turned_diagonal_matrix() {
  let turn: DMat3 = DMat3::from_quat(DQuat::from_axis_angle(DVec3::new(1.0, -2.0, 0.5).normalize(), 0.9));
  let matrix: DMat3 = turn * DMat3::from_diagonal(DVec3::new(4.0, 1.0, 0.25)) * turn.transpose();
  let rows: [[f64; 3]; 3] = matrix.transpose().to_cols_array_2d();
  let eigen: SymmetricEigen = SymmetricEigen::of(rows);
  let mut values: [f64; 3] = eigen.values;

  values.sort_by(f64::total_cmp);

  for (value, expected) in values.iter().zip([0.25, 1.0, 4.0]) {
    assert!((value - expected).abs() < 1e-12, "{values:?}");
  }

  for (value, vector) in eigen.values.iter().zip(eigen.vectors) {
    assert!((vector.length() - 1.0).abs() < 1e-12);
    assert!(
      (matrix * vector - vector * *value).length() < 1e-12,
      "{vector} is not an axis"
    );
  }

  assert!((eigen.vectors[eigen.get_greatest()].dot(turn.x_axis).abs() - 1.0).abs() < 1e-12);
  assert!((eigen.vectors[eigen.get_least()].dot(turn.z_axis).abs() - 1.0).abs() < 1e-12);
}

#[test]
fn settles_alike_at_any_scale() {
  let rows: [[f64; 3]; 3] = [[2.0, 0.5, 0.1], [0.5, 1.0, 0.3], [0.1, 0.3, 0.5]];
  let small: SymmetricEigen = SymmetricEigen::of(rows.map(|row| row.map(|value| value * 1e-6)));
  let large: SymmetricEigen = SymmetricEigen::of(rows.map(|row| row.map(|value| value * 1e6)));

  for (small, large) in small.vectors.iter().zip(large.vectors) {
    assert!((small.dot(large).abs() - 1.0).abs() < 1e-12, "{small} against {large}");
  }
}

#[test]
fn keeps_a_diagonal_matrix_as_it_is() {
  let eigen: SymmetricEigen = SymmetricEigen::of([[3.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 1.0]]);

  assert_eq!(eigen.values, [3.0, 3.0, 1.0]);
  assert_eq!(eigen.vectors, [DVec3::X, DVec3::Y, DVec3::Z]);
  assert_eq!((eigen.get_least(), eigen.get_greatest()), (2, 1));
}
