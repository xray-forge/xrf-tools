use xrf_renderer_core::ShaderStruct;

/// What the scene pass reads the water's every-pixel rays by (`WaterRays`, `common/water_rays`): the metres they are
/// traced, the share of the sky's picture the water reflects where they meet nothing (`WaterUniform::reflection`), and
/// the frame they were written on, whose low half each record keeps.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "WaterRaysFrame")]
pub(crate) struct WaterRaysUniform {
  pub trace_distance: f32,
  pub sky_share: f32,
  pub frame: u32,
}
