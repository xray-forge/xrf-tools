use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::grass_colour_parameters::GrassColourParameters;
use crate::pass::pipeline_source::PipelineSource;

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
  pub fn new(source: PipelineSource<'_>, texture_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = GrassColourParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, &layout, texture_layout)?,
      layout,
      texture_layout: texture_layout.clone(),
      generation: source.get_generation(),
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.layout, &self.texture_layout) {
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
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("grass colours", "grass/colours", "cs_grass_colours")
        .with_bind_group_layouts(&[Some(layout), Some(texture_layout)]),
    )
  }
}
