use glam::Vec4;

/// Where something stands: its matrix in four columns, then what lights it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticPlace {
  pub columns: [Vec4; 4],
  /// The hemisphere's scale and bias, the impostor it stands for or `-1`, and the matrix's largest axis scale.
  pub info: Vec4,
  /// A dynamic object's hemisphere cube, packed; `w` above a half marks it present.
  pub cube: Vec4,
}

impl Default for StaticPlace {
  fn default() -> Self {
    Self {
      columns: [Vec4::X, Vec4::Y, Vec4::Z, Vec4::W],
      info: Vec4::new(1.0, 0.0, -1.0, 1.0),
      cube: Vec4::ZERO,
    }
  }
}
