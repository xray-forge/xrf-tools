use glam::{Mat4, Vec3};

use crate::camera::camera_view::CameraView;

/// Where a viewport's camera stands this frame and what its lens is, as the world's controller left it; the renderer
/// fits it to the viewport's shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraFrame {
  pub position: Vec3,
  /// From the world into the camera's space.
  pub view: Mat4,
  /// The lens: vertical field of view in degrees, and the near and far planes.
  pub field_of_view: f32,
  pub near: f32,
  pub far: f32,
}

impl Default for CameraFrame {
  fn default() -> Self {
    Self {
      position: Vec3::ZERO,
      view: Mat4::IDENTITY,
      field_of_view: 60.0,
      near: 0.1,
      far: 1000.0,
    }
  }
}

impl CameraFrame {
  /// The view through the lens at a viewport's aspect, its far plane no farther than `far_limit` and past the near one.
  pub fn to_view(&self, aspect: f32, far_limit: f32) -> CameraView {
    CameraView::new(
      self.position,
      self.view,
      self.field_of_view,
      aspect,
      self.near,
      self.far.min(far_limit).max(self.near * 2.0),
    )
  }
}
