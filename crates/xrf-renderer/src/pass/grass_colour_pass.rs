use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters};

use crate::pass::grass_colour_parameters::GrassColourParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;

/// Averages each detail model's texture over its solid texels into the colour a traced hit on grassed ground blends
/// towards (`grass/colours`), a workgroup a model, each frame the scene is traced: a few thousand reads a model, and
/// always the textures as they stand, streamed in or not.
pub struct GrassColourPass {
  layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  generation: u64,
}

impl GrassColourPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = GrassColourParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, source, (&layout, texture_layout))?,
      layout,
      texture_layout: texture_layout.clone(),
      generation: source.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(device, source, (&self.layout, &self.texture_layout)) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Grass colours rejected, averaging with the last: {error}"),
      }
    }
  }

  /// Averages the colours of `models` detail models, their textures read through the texture cache's group.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    parameters: &GrassColourParameters,
    textures: &wgpu::BindGroup,
    models: u32,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(1, textures, &[]);
    pass.dispatch_workgroups(models, 1, 1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    (layout, texture_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
  ) -> XrfResult<wgpu::ComputePipeline> {
    let module: wgpu::ShaderModule = source.get_module("grass/colours")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("grass colours"),
      bind_group_layouts: &[Some(layout), Some(texture_layout)],
      ..Default::default()
    });

    create_checked(device, "cs_grass_colours", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("grass colours"),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("cs_grass_colours"),
        compilation_options: Default::default(),
        cache: source.get_driver_cache(),
      })
    })
  }
}
