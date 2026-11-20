use glam::DVec3;
use xrf_math::{Affine3d, Vector3d};

/// A part's box (`LevelBuildParts::fit_box`): its centre, its axes, orthonormal and right-handed, and its half size
/// along each.
pub(crate) struct LevelBuildPartBox {
  pub(crate) centre: DVec3,
  pub(crate) axes: [DVec3; 3],
  pub(crate) half: DVec3,
}

impl LevelBuildPartBox {
  /// The box's frame in single precision, its axes as the rows a spawn packet's shape and placement take.
  pub(crate) fn get_frame(&self) -> Affine3d {
    Affine3d {
      x_axis: Vector3d::from(self.axes[0].as_vec3()),
      y_axis: Vector3d::from(self.axes[1].as_vec3()),
      z_axis: Vector3d::from(self.axes[2].as_vec3()),
      translation: Vector3d::from(self.centre.as_vec3()),
    }
  }

  /// How far `vector` runs along each of the box's axes; a point is first taken from the centre by the caller.
  pub(crate) fn get_local(&self, vector: DVec3) -> DVec3 {
    DVec3::from_array(self.axes.map(|axis| axis.dot(vector)))
  }
}
