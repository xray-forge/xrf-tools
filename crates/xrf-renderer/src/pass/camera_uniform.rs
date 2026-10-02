use glam::{Mat4, Vec4};

use crate::camera::camera_view::CameraView;

/// The camera as `shaders/common/camera.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  pub view_projection: Mat4,
  pub inverse_view_projection: Mat4,
  pub position: Vec4,
  pub viewport: Vec4,
}

impl CameraUniform {
  pub fn new(view: &CameraView, width: u32, height: u32) -> Self {
    let view_projection: Mat4 = view.get_view_projection();

    Self {
      view_projection,
      inverse_view_projection: view_projection.inverse(),
      position: view.position.extend(1.0),
      viewport: Vec4::new(width as f32, height as f32, 0.0, 0.0),
    }
  }
}
