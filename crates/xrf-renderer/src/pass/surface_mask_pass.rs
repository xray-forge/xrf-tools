use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters};

use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::surface_heights_parameters::SurfaceHeightsParameters;
use crate::pass::surface_sites_parameters::SurfaceSitesParameters;
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a workgroup runs a side, as `shaders/frame/surface_mask.wgsl` declares them.
const WORKGROUP: u32 = 8;

/// Places the puddles on the level's surface seen from overhead: its lowest heights, then a site a cell, a dispatch
/// each.
pub struct SurfaceMaskPass {
  layouts: [wgpu::BindGroupLayout; 2],
  /// The lowest heights and the sites.
  pipelines: [wgpu::ComputePipeline; 2],
  generation: u64,
}

impl SurfaceMaskPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let layouts: [wgpu::BindGroupLayout; 2] = [
      SurfaceHeightsParameters::create_layout(device),
      SurfaceSitesParameters::create_layout(device),
    ];

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layouts)?,
      generation: shaders.get_generation(),
      layouts,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Puddle sites rejected, placing with the last ones: {error}"),
      }
    }
  }

  /// Takes the lowest heights over a map `size` texels a side, then places a site in each of `cells` a side.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    (size, cells): (u32, u32),
    (lowest, sites): (&SurfaceHeightsParameters, &SurfaceSitesParameters),
  ) {
    context.bind(lowest);
    context.get_pass().set_pipeline(&self.pipelines[0]);
    context
      .get_pass()
      .dispatch_workgroups(size.div_ceil(WORKGROUP), size.div_ceil(WORKGROUP), 1);
    context.bind(sites);
    context.get_pass().set_pipeline(&self.pipelines[1]);
    context
      .get_pass()
      .dispatch_workgroups(cells.div_ceil(WORKGROUP), cells.div_ceil(WORKGROUP), 1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    [heights, sites]: &[wgpu::BindGroupLayout; 2],
  ) -> XrfResult<[wgpu::ComputePipeline; 2]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/surface_mask")?;
    let create = |entry: &str, layout: &wgpu::BindGroupLayout| -> XrfResult<wgpu::ComputePipeline> {
      let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(entry),
        bind_group_layouts: &[Some(layout)],
        ..Default::default()
      });

      create_checked(device, entry, || {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
          label: Some(entry),
          layout: Some(&pipeline_layout),
          module: &module,
          entry_point: Some(entry),
          compilation_options: Default::default(),
          cache: None,
        })
      })
    };

    Ok([create("surface_lowest", heights)?, create("surface_sites", sites)?])
  }
}
