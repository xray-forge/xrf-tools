use xrf_renderer_core::ShaderStruct;

/// How the scene pass reads the water's every-pixel rays (`WaterRays`, `common/water_rays`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "WaterRaysFrame")]
pub(crate) struct WaterRaysUniform {
  /// Metres the rays are traced.
  pub trace_distance: f32,
  /// The share of the sky's picture the water reflects where a ray meets nothing (`WaterUniform::reflection`).
  pub sky_share: f32,
  /// The frame the rays were written on, whose low half each record keeps.
  pub frame: u32,
}
