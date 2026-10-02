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
  /// Textured, bumped, the baked hemisphere's strength.
  pub switches: Vec4,
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
      planes: to_frustum_planes(view_projection),
      switches,
    }
  }
}

/// The six planes of a view projection into zero-to-one depth, each normalized so a sphere tests by its radius.
fn to_frustum_planes(matrix: Mat4) -> [Vec4; 6] {
  let rows: [Vec4; 4] = [matrix.row(0), matrix.row(1), matrix.row(2), matrix.row(3)];

  [
    rows[3] + rows[0],
    rows[3] - rows[0],
    rows[3] + rows[1],
    rows[3] - rows[1],
    rows[2],
    rows[3] - rows[2],
  ]
  .map(|plane| plane / plane.truncate().length())
}
