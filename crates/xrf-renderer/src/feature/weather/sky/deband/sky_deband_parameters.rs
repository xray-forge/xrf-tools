use xrf_renderer_core::{GraphTexture, PassParameters};

/// What smoothing a sky cube reads and writes: the cube as the weather holds it, its sampler, and the copy's six faces.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct SkyDebandCubeParameters<'a> {
  #[texture(cube, float)]
  pub cube: GraphTexture,
  #[sampler(filtering)]
  pub cube_sampler: &'a wgpu::Sampler,
  #[storage_texture(d2_array, rgba16float, write)]
  pub faces: GraphTexture,
}

/// What smoothing a cloud texture reads and writes: the clouds as the weather holds them, their repeating sampler, and
/// the copy.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct SkyDebandFlatParameters<'a> {
  #[texture(d2, float)]
  pub clouds: GraphTexture,
  #[sampler(filtering)]
  pub clouds_sampler: &'a wgpu::Sampler,
  #[storage_texture(d2, rgba16float, write)]
  pub smoothed: GraphTexture,
}
