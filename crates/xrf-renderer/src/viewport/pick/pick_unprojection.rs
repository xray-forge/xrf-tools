use glam::Vec2;

use crate::camera::camera_view::CameraView;

/// How a pick's depth unprojects: the camera it looked through, which its depth is measured along, and its point on the
/// screen in normalised coordinates.
#[derive(Clone, Copy)]
pub struct PickUnprojection {
  pub view: CameraView,
  pub ndc: Vec2,
}
