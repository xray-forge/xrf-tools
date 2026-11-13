use xrf_math::{EPS, Vector3d};

/// A face's area as the Level Editor's Build and xrLC measure it before they refuse it: `CalcArea`
/// (`BuilderRemote.cpp:509`, `xrLC_Light/MeshStructure.h:192`), Heron's formula over the edges in single precision.
pub struct FaceArea;

impl FaceArea {
  /// The area of the triangle `corners`, `NaN` where rounding leaves the product under the root negative.
  pub fn of([a, b, c]: &[Vector3d; 3]) -> f32 {
    let (first, second, third) = (a.distance(b), a.distance(c), b.distance(c));
    let half: f32 = (first + second + third) / 2.0;

    (half * (half - first) * (half - second) * (half - third)).sqrt()
  }

  /// Whether the face has the area the Build keeps it for and xrLC asks of it (`BuilderRemote.cpp:680`,
  /// `xrLC_Light/xrFace.cpp:198`): a number, no less than [`EPS`].
  pub fn is_enough(corners: &[Vector3d; 3]) -> bool {
    let area: f32 = Self::of(corners);

    area.is_finite() && area >= EPS
  }
}
