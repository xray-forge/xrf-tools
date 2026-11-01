use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What resolving one face of the surroundings cube reads as its `Surroundings`: towards the sun in renderer space,
/// then the face, the cube's texels a side and the face's G-buffer's texels a side.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Surroundings")]
pub struct SurroundingsUniform {
  pub to_sun: Vec4,
  pub face: Vec4,
}

impl SurroundingsUniform {
  /// Face `face` of a cube `size` texels a side, drawn into a G-buffer `drawn` texels a side, lit by a sun whose light
  /// comes from `to_sun`.
  pub fn new(to_sun: glam::Vec3, face: u32, (size, drawn): (u32, u32)) -> Self {
    Self {
      to_sun: to_sun.extend(0.0),
      face: Vec4::new(face as f32, size as f32, drawn as f32, 0.0),
    }
  }
}
