use glam::{Mat4, UVec4, Vec4};
use xrf_renderer_core::ShaderStruct;

/// Where something stands: its matrix, then what lights it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Place")]
pub struct StaticPlace {
  pub transform: Mat4,
  /// The hemisphere's scale and bias, the impostor it stands for or `-1`, and the matrix's largest axis scale.
  pub info: Vec4,
  /// A spawned object's hemisphere cube, its six faces (`+x +y +z -x -y -z`) as bytes in `x` and `y`; `w` one where it
  /// has one.
  pub cube: UVec4,
  /// A skinned model's: its bone matrices' first row, its links' first vertex, its own first vertex in the model arena,
  /// and how many bones it has, zero where it is rigid. The last frame's matrices follow this frame's.
  pub skin: UVec4,
}

impl Default for StaticPlace {
  fn default() -> Self {
    Self {
      transform: Mat4::IDENTITY,
      info: Vec4::new(1.0, 0.0, -1.0, 1.0),
      cube: UVec4::ZERO,
      skin: UVec4::ZERO,
    }
  }
}
