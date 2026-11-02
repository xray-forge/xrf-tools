use xrf_renderer_core::PassParameters;

use crate::feature::weather::water::draw::water_depth_parameters::WaterDepthParameters;
use crate::feature::weather::water::draw::water_surface_parameters::WaterSurfaceParameters;
use crate::feature::weather::water::enhanced::enhanced_water_parameters::EnhancedWaterParameters;
use crate::feature::weather::water::reflection::water_reflection_parameters::WaterReflectionParameters;

/// The water passes' own groups' layouts, made once: a pipeline made again on a reload keys the cache as the last did.
pub struct WaterParameterLayouts {
  pub engine: wgpu::BindGroupLayout,
  pub enhanced: wgpu::BindGroupLayout,
  pub depth: wgpu::BindGroupLayout,
  pub reflection: wgpu::BindGroupLayout,
}

impl WaterParameterLayouts {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      engine: WaterSurfaceParameters::create_layout(device),
      enhanced: EnhancedWaterParameters::create_layout(device),
      depth: WaterDepthParameters::create_layout(device),
      reflection: WaterReflectionParameters::create_layout(device),
    }
  }
}
