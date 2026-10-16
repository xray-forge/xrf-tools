use glam::Vec4;

/// A view's six planes in renderer space, pointing in, each normalized so a sphere tests by its radius: made once to
/// test many spheres.
#[derive(Clone, Copy, Debug)]
pub struct CameraFrustum {
  pub planes: [Vec4; 6],
}

impl CameraFrustum {
  /// Whether any of a sphere in the world, its centre and radius, falls within the frustum.
  pub fn reaches(&self, sphere: Vec4) -> bool {
    self
      .planes
      .iter()
      .all(|plane| plane.truncate().dot(sphere.truncate()) + plane.w >= -sphere.w)
  }
}
