use glam::Vec4;

/// Where something stands: its matrix in four columns, then what lights it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticPlace {
  pub columns: [Vec4; 4],
  /// The hemisphere's scale and bias, the impostor it stands for or `-1`, and the matrix's largest axis scale.
  pub info: Vec4,
  /// A spawned object's hemisphere cube, its six faces (`+x +y +z -x -y -z`) as bytes in `x` and `y`; `w` one where it
  /// has one.
  pub cube: [u32; 4],
  /// A skinned model's: its bone matrices' first row, its links' first vertex, its own first vertex in the model arena,
  /// and how many bones it has, zero where it is rigid. The last frame's matrices follow this frame's.
  pub skin: [u32; 4],
}

impl Default for StaticPlace {
  fn default() -> Self {
    Self {
      columns: [Vec4::X, Vec4::Y, Vec4::Z, Vec4::W],
      info: Vec4::new(1.0, 0.0, -1.0, 1.0),
      cube: [0; 4],
      skin: [0; 4],
    }
  }
}
