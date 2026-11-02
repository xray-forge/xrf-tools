use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::camera::camera_frustum::CameraFrustum;

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

  /// A right-handed view through a lens of parallel rays, `half_size` across either way of its centre, with reversed
  /// depth as the perspective's: what a level map is drawn through.
  pub fn new_orthographic(
    position: Vec3,
    view: Mat4,
    (half_width, half_height): (f32, f32),
    near: f32,
    far: f32,
  ) -> Self {
    Self {
      position,
      view,
      projection: glam::camera::rh::proj::directx::orthographic(
        -half_width,
        half_width,
        -half_height,
        half_height,
        far,
        near,
      ),
    }
  }

  /// Whether its lens has parallel rays: a projection leaving `w` as it is.
  pub fn is_orthographic(&self) -> bool {
    self.projection.w_axis.w == 1.0
  }

  /// Its lens's height in degrees, read back from the projection; none for parallel rays, which do not spread.
  pub fn get_field_of_view(&self) -> Option<f32> {
    (!self.is_orthographic()).then(|| (2.0 * (1.0 / self.projection.y_axis.y).atan()).to_degrees())
  }

  /// The tangents of its lens's half view across and up, read back from the projection; none for parallel rays, which
  /// reach the film square on.
  pub fn get_lens_tangents(&self) -> Vec2 {
    if self.is_orthographic() {
      return Vec2::ZERO;
    }

    Vec2::new(1.0 / self.projection.x_axis.x, 1.0 / self.projection.y_axis.y)
  }

  /// The box a lens of parallel rays sees, across, up and deep, read back from the projection; none for a perspective.
  pub fn get_orthographic_extent(&self) -> Option<Vec3> {
    self.is_orthographic().then(|| {
      let (near, far): (f32, f32) = self.get_depth_range();

      Vec3::new(
        2.0 / self.projection.x_axis.x,
        2.0 / self.projection.y_axis.y,
        far - near,
      )
    })
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

  /// The same view with its samples moved within their pixels, by `offset` drawn pixels, `y` down, across a drawing
  /// `size` pixels big: through the clip offset a perspective carries times `w` in its third column, which is `-z`, or
  /// the one an orthographic lens carries in its fourth, its `w` being one.
  pub fn jittered(&self, offset: Vec2, size: Vec2) -> Self {
    let mut projection: Mat4 = self.projection;
    let shift: Vec2 = Vec2::new(2.0 * offset.x / size.x.max(1.0), 2.0 * offset.y / size.y.max(1.0));

    if self.is_orthographic() {
      projection.w_axis.x -= shift.x;
      projection.w_axis.y += shift.y;
    } else {
      projection.z_axis.x += shift.x;
      projection.z_axis.y -= shift.y;
    }

    Self {
      position: self.position,
      view: self.view,
      projection,
    }
  }

  pub fn get_view_projection(&self) -> Mat4 {
    self.projection * self.view
  }

  /// The near and far distances a reversed lens was made with, read back from its depth terms.
  pub fn get_depth_range(&self) -> (f32, f32) {
    let scale: f32 = self.projection.z_axis.z;

    if self.is_orthographic() {
      // Depth is `(far - d) / (far - near)`: a scale of `1 / (far - near)` and an offset of `far` times it.
      let far: f32 = self.projection.w_axis.z / scale;

      return (far - 1.0 / scale, far);
    }

    let far: f32 = self.projection.w_axis.z / scale;

    (scale * far / (1.0 + scale), far)
  }

  /// The view's frustum, to test spheres against.
  pub fn get_frustum(&self) -> CameraFrustum {
    CameraFrustum {
      planes: self.get_planes(),
    }
  }

  /// The direction the view looks, in renderer space.
  pub fn get_look(&self) -> Vec3 {
    (-self.view.inverse().z_axis.truncate()).normalize_or_zero()
  }

  /// The view's six planes in renderer space, pointing in, each normalized so a sphere tests by its radius.
  pub fn get_planes(&self) -> [Vec4; 6] {
    let matrix: Mat4 = self.get_view_projection();
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
}
