use glam::{Mat4, Vec2, Vec3, Vec4};

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

  /// The same view through a lens narrowed to a rectangle of it, centred at `center` and `size` across in normalized
  /// device coordinates: what a pick draws its one texel through.
  pub fn narrow_to(&self, center: Vec2, size: Vec2) -> Self {
    let scale: Vec2 = Vec2::new(2.0 / size.x, 2.0 / size.y);
    let narrowing: Mat4 = Mat4::from_cols(
      Vec4::new(scale.x, 0.0, 0.0, 0.0),
      Vec4::new(0.0, scale.y, 0.0, 0.0),
      Vec4::new(0.0, 0.0, 1.0, 0.0),
      Vec4::new(-center.x * scale.x, -center.y * scale.y, 0.0, 1.0),
    );

    Self {
      position: self.position,
      view: self.view,
      projection: narrowing * self.projection,
    }
  }

  pub fn get_view_projection(&self) -> Mat4 {
    self.projection * self.view
  }
}
