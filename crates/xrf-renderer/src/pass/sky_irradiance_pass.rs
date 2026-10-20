use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sky_irradiance_parameters::SkyIrradianceParameters;

/// Projects the sky's irradiance into spherical harmonics for the physical light, one workgroup a frame.
pub struct SkyIrradiancePass {
  layout: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  generation: u64,
}

impl SkyIrradiancePass {
  /// Invocations the workgroup runs, as `shaders/frame/sky_irradiance.wgsl` declares them.
  pub(crate) const WORKGROUP: u32 = 64;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SkyIrradianceParameters::create_layout(source.device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, &layout)?,
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipeline again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last one where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sky irradiance rejected, projecting with the last one: {error}"),
      }
    }
  }

  /// Projects the sky once.
  pub fn record(&self, context: &mut ComputeContext<'_>, parameters: &SkyIrradianceParameters<'_>) {
    context.bind(parameters);
    context.get_pass().set_pipeline(&self.pipeline);
    context.get_pass().dispatch_workgroups(1, 1, 1);
  }

  fn create_pipeline(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("sky irradiance", "frame/sky_irradiance", "cs_irradiance")
        .with_bind_group_layouts(&[Some(layout)]),
    )
  }
}
