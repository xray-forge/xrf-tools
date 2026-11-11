/// Vectors in double precision, for the fits a build ports from the editor's geometry code: a portal's plane, a part's
/// box.
pub(crate) struct LevelBuildVector;

impl LevelBuildVector {
  pub(crate) fn add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
  }

  pub(crate) fn sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
  }

  pub(crate) fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
  }

  pub(crate) fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
      a[1] * b[2] - a[2] * b[1],
      a[2] * b[0] - a[0] * b[2],
      a[0] * b[1] - a[1] * b[0],
    ]
  }

  pub(crate) fn length(a: &[f64; 3]) -> f64 {
    Self::dot(a, a).sqrt()
  }

  /// `a` at unit length; none for a vector of no length.
  pub(crate) fn normalize(a: &[f64; 3]) -> Option<[f64; 3]> {
    let length: f64 = Self::length(a);

    (length > f64::EPSILON).then(|| a.map(|value| value / length))
  }
}
