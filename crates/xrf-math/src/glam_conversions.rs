//! The vectors as `glam`'s and back, for the runtime crates computing with `glam` over what a format stored: the
//! components carry across unchanged, in no other space than the one they are in.

use glam::{Vec2, Vec3};

use crate::vector_2d::Vector2d;
use crate::vector_3d::Vector3d;

impl From<&Vector3d> for Vec3 {
  fn from(vector: &Vector3d) -> Self {
    Self::new(vector.x, vector.y, vector.z)
  }
}

impl From<Vector3d> for Vec3 {
  fn from(vector: Vector3d) -> Self {
    Self::from(&vector)
  }
}

impl From<Vec3> for Vector3d {
  fn from(vector: Vec3) -> Self {
    Self::new(vector.x, vector.y, vector.z)
  }
}

impl From<Vector2d> for Vec2 {
  fn from(vector: Vector2d) -> Self {
    Self::new(vector.x, vector.y)
  }
}

impl From<Vec2> for Vector2d {
  fn from(vector: Vec2) -> Self {
    Self::new(vector.x, vector.y)
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec2, Vec3};

  use crate::vector_2d::Vector2d;
  use crate::vector_3d::Vector3d;

  #[test]
  fn carries_each_component_across_and_back() {
    let stored: Vector3d = Vector3d::new(1.5, -2.0, 3.25);

    assert_eq!(Vec3::from(&stored), Vec3::new(1.5, -2.0, 3.25));
    assert_eq!(Vector3d::from(Vec3::from(stored)), stored);
    assert_eq!(
      Vector2d::from(Vec2::from(Vector2d::new(0.25, 4.0))),
      Vector2d::new(0.25, 4.0)
    );
  }
}
