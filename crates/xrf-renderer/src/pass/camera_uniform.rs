use glam::{Mat4, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;

/// The camera as `shaders/common/camera.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  pub view_projection: Mat4,
  pub inverse_view_projection: Mat4,
  pub view: Mat4,
  pub projection: Mat4,
  pub inverse_projection: Mat4,
  pub position: Vec4,
  /// The viewport's size, then its top left corner in the window.
  pub viewport: Vec4,
  /// The frustum's planes in renderer space, pointing inward.
  pub planes: [Vec4; 6],
  /// Textured, bumped, the baked hemisphere's strength, how far the water distorts what is behind it.
  pub switches: Vec4,
  /// One where the static surfaces draw as their triangles' edges.
  pub modes: Vec4,
  /// World to clip without the jitter, this frame and the last, which a surface's motion is measured by.
  pub motion_current: Mat4,
  pub motion_previous: Mat4,
}

impl CameraUniform {
  pub fn new(view: &CameraView, rect: RenderRect, switches: Vec4) -> Self {
    let view_projection: Mat4 = view.get_view_projection();

    Self {
      view_projection,
      inverse_view_projection: view_projection.inverse(),
      view: view.view,
      projection: view.projection,
      inverse_projection: view.projection.inverse(),
      position: view.position.extend(1.0),
      viewport: Vec4::new(rect.width as f32, rect.height as f32, rect.x as f32, rect.y as f32),
      planes: view.get_planes(),
      switches,
      modes: Vec4::ZERO,
      motion_current: view_projection,
      motion_previous: view_projection,
    }
  }

  /// The same camera measuring motion between two unjittered view projections, the last frame's and this one's.
  pub fn with_motion(mut self, (current, previous): (Mat4, Mat4)) -> Self {
    self.motion_current = current;
    self.motion_previous = previous;
    self
  }

  /// The same camera drawing every static surface as its triangles' edges, or as itself.
  pub fn with_wireframe(mut self, is_wireframe: bool) -> Self {
    self.modes.x = f32::from(u8::from(is_wireframe));
    self
  }
}
