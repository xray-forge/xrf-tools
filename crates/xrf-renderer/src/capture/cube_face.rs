use glam::{Mat4, Vec3};

use crate::camera::camera_view::CameraView;
use crate::camera::engine_space::EngineSpace;

/// Each face's direction and up in engine space, in the engine's order `+x -x +y -y +z -z` (`CDemoRecord`'s `cmDir`
/// and `cmNorm`): the layout a cube texture lays its faces' texels by.
const FACES: [(Vec3, Vec3); 6] = [
  (Vec3::X, Vec3::Y),
  (Vec3::NEG_X, Vec3::Y),
  (Vec3::Y, Vec3::NEG_Z),
  (Vec3::NEG_Y, Vec3::Z),
  (Vec3::Z, Vec3::Y),
  (Vec3::NEG_Z, Vec3::Y),
];

/// One face of a cubemap: which, where it is seen from and its lens's depth, and the pixels it is drawn at.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubeFace {
  /// Its place in the engine's order `+x -x +y -y +z -z`, from zero.
  pub index: usize,
  /// Where it is seen from, renderer space.
  pub position: Vec3,
  /// Its near and far planes.
  pub near: f32,
  pub far: f32,
  /// Its pixels on a side.
  pub size: u32,
  /// Times over on each side it is drawn.
  pub supersampling: u32,
}

impl CubeFace {
  /// Faces a cube has.
  pub const COUNT: usize = FACES.len();

  /// A face's lens, the quarter turn that makes six of them a cube.
  pub const FIELD_OF_VIEW: f32 = 90.0;

  /// The pixels it is drawn at on a side, times its supersampling.
  pub fn get_drawn_size(&self) -> (u32, u32) {
    let side: u32 = self.size * self.supersampling;

    (side, side)
  }

  /// Its view, renderer space: 90 degrees square down its axis, its up as a cube texture lays its face.
  pub fn get_view(&self) -> CameraView {
    let (direction, up): (Vec3, Vec3) = FACES[self.index % Self::COUNT];
    let look: Mat4 =
      glam::camera::rh::view::look_to_mat4(self.position, EngineSpace::mirror(direction), EngineSpace::mirror(up));

    CameraView::new(self.position, look, Self::FIELD_OF_VIEW, 1.0, self.near, self.far)
  }
}
