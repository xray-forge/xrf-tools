use xrf_math::glam;
use xrf_math::glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What resolving one face of the surroundings cube reads as its `Surroundings`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Surroundings")]
pub(crate) struct SurroundingsUniform {
  /// Towards the sun, in renderer space.
  pub to_sun: Vec4,
  /// The face, the cube's texels a side, the G-buffer's texels a side, and the face's layer of the G-buffer arrays.
  pub face: Vec4,
}

impl SurroundingsUniform {
  /// Face `face` of a cube `size` texels a side, drawn into `layer` of G-buffer arrays `drawn` texels a side.
  pub(crate) fn new(to_sun: glam::Vec3, (face, layer): (u32, u32), (size, drawn): (u32, u32)) -> Self {
    Self {
      to_sun: to_sun.extend(0.0),
      face: Vec4::new(face as f32, size as f32, drawn as f32, layer as f32),
    }
  }
}
