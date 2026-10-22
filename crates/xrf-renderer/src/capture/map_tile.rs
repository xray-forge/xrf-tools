use glam::{Mat4, Vec3};

use crate::camera::camera_view::CameraView;
use crate::camera::engine_space::EngineSpace;
use crate::contract::render_rect::RenderRect;

/// The height the camera stands above a capture's top, its near plane there, so nothing at the top is cut by it.
const EYE_LIFT: f32 = 1.0;

/// One tile of a level map: the map pixels it fills and the area it is drawn over, a margin past each side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapTile {
  /// Where it is put in the map, in map pixels from its top left corner.
  pub placed: RenderRect,
  /// Map pixels it is drawn past each side of `placed`, cut off when it is put in.
  pub margin: u32,
  /// Times over on each side it is drawn.
  pub supersampling: u32,
  /// The area it is drawn over in engine space, margin included: least and greatest `x`, then `z`.
  pub min_x: f32,
  pub max_x: f32,
  pub min_z: f32,
  pub max_z: f32,
  /// The heights it is drawn between, engine `y`.
  pub bottom: f32,
  pub top: f32,
  /// Whether the grass is planted over it.
  pub is_grassed: bool,
}

impl MapTile {
  /// The pixels it is drawn at, across and down: its margin included, times its supersampling.
  pub fn get_drawn_size(&self) -> (u32, u32) {
    (
      (self.placed.width + 2 * self.margin) * self.supersampling,
      (self.placed.height + 2 * self.margin) * self.supersampling,
    )
  }

  /// Its view straight down onto its area, renderer space, right engine `+x` and up engine `+z` as `CDemoRecord`'s.
  pub fn get_view(&self) -> CameraView {
    let center: Vec3 = EngineSpace::mirror(Vec3::new(
      (self.min_x + self.max_x) * 0.5,
      self.top + EYE_LIFT,
      (self.min_z + self.max_z) * 0.5,
    ));
    let up: Vec3 = EngineSpace::mirror(Vec3::Z);
    let view: Mat4 = glam::camera::rh::view::look_to_mat4(center, Vec3::NEG_Y, up);
    let (half_width, half_height): (f32, f32) = ((self.max_x - self.min_x) * 0.5, (self.max_z - self.min_z) * 0.5);

    CameraView::new_orthographic(
      center,
      view,
      (half_width, half_height),
      EYE_LIFT,
      EYE_LIFT + (self.top - self.bottom).max(EYE_LIFT),
    )
  }
}
