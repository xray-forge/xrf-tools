use crate::camera::camera_view::CameraView;
use crate::contract::feature::image::render_camera_settings::RenderCameraSettings;

/// What the camera draws a frame with: its settings and the view it looks through.
#[derive(Clone, Copy, Debug)]
pub struct CameraFrame {
  pub settings: RenderCameraSettings,
  pub view: CameraView,
}
