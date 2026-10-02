use glam::{Mat4, Vec3};

/// What a camera sees from where it stands this frame, in renderer space.
#[derive(Clone, Copy, Debug)]
pub struct CameraView {
  pub position: Vec3,
  pub view: Mat4,
  pub projection: Mat4,
}

impl CameraView {
  /// A right-handed view through a perspective lens with reversed depth: one at the near plane, zero at the far one,
  /// which spends a float depth buffer's precision where a level needs it.
  pub fn new(position: Vec3, view: Mat4, field_of_view: f32, aspect: f32, near: f32, far: f32) -> Self {
    Self {
      position,
      view,
      // Swapping near and far is what reverses the depth of a right-handed, zero-to-one perspective.
      projection: glam::camera::rh::proj::directx::perspective(field_of_view.to_radians(), aspect.max(1e-6), far, near),
    }
  }

  pub fn get_view_projection(&self) -> Mat4 {
    self.projection * self.view
  }
}
