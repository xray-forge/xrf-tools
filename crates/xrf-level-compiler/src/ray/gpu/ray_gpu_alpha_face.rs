use xrf_math::Vector2d;

/// A face dimming light through alpha, as the shader reads it (`AlphaFace`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RayGpuAlphaFace {
  corners: [[f32; 2]; 3],
  texture: u32,
  _pad: u32,
}

impl RayGpuAlphaFace {
  /// A face of texture `texture` with corners at `coordinates`, as `ShadowMaterials::get_alpha_faces` gives it.
  pub(crate) fn of((texture, coordinates): (u32, [Vector2d; 3])) -> Self {
    Self {
      corners: coordinates.map(|corner| [corner.x, corner.y]),
      texture,
      _pad: 0,
    }
  }
}
