use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters};

use crate::frame::depth_pyramid::DepthPyramid;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a reduction workgroup runs a side, as `shaders/frame/pyramid.wgsl` declares them.
const WORKGROUP: u32 = 8;

/// Reduces a viewport's depth into a pyramid, a dispatch a level: to the farthest under each texel, or the nearest.
pub struct DepthPyramidPass {
  depth_layout: wgpu::BindGroupLayout,
  level_layout: wgpu::BindGroupLayout,
  /// Reducing the depth into the first level, then a level into the next: the farthest's pair, then the nearest's.
  pipelines: [wgpu::ComputePipeline; 4],
  generation: u64,
}

impl DepthPyramidPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let depth_layout: wgpu::BindGroupLayout = PyramidDepthParameters::create_layout(device);
    let level_layout: wgpu::BindGroupLayout = PyramidLevelParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &depth_layout, &level_layout)?,
      generation: shaders.get_generation(),
      depth_layout,
      level_layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.depth_layout, &self.level_layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Depth pyramid rejected, reducing with the last one: {error}"),
      }
    }
  }

  /// Reduces the depth into the first level, then each level into the next, a dispatch each: to the nearest where
  /// `is_nearest`, the farthest otherwise.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    pyramid: &DepthPyramid,
    (first, levels): (&PyramidDepthParameters, &[PyramidLevelParameters]),
    is_nearest: bool,
  ) {
    for level in 0..pyramid.levels as usize {
      let (width, height): (u32, u32) = pyramid.get_level_size(level as u32);

      match level {
        0 => context.bind(first),
        _ => context.bind(&levels[level - 1]),
      }

      let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

      pass.set_pipeline(&self.pipelines[usize::from(is_nearest) * 2 + usize::from(level > 0)]);
      pass.dispatch_workgroups(width.div_ceil(WORKGROUP), height.div_ceil(WORKGROUP), 1);
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    depth_layout: &wgpu::BindGroupLayout,
    level_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::ComputePipeline; 4]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/pyramid")?;
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

    Ok([
      create("reduce_depth", depth_layout)?,
      create("reduce_level", level_layout)?,
      create("copy_nearest", depth_layout)?,
      create("reduce_nearest", level_layout)?,
    ])
  }
}
