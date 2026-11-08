use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters, PipelineSet};

use crate::feature::weather::sky::irradiance::sky_irradiance_parameters::SkyIrradianceParameters;
use crate::pass::build::pipeline_source::PipelineSource;

/// Projects the sky's irradiance into spherical harmonics for the physical light, one workgroup a frame.
pub struct SkyIrradiancePass {
  layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::ComputePipeline>,
}

impl SkyIrradiancePass {
  /// Invocations the workgroup runs, as `shaders/frame/sky_irradiance.wgsl` declares them.
  #[cfg(test)]
  pub(crate) const WORKGROUP: u32 = 64;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SkyIrradianceParameters::create_layout(source.device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, &layout)?),
      layout,
    })
  }

  /// Makes its pipeline again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last one where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self
      .pipeline
      .refresh(source.cache, || Self::create_pipeline(source, &self.layout))
    {
      log::error!("Sky irradiance rejected, projecting with the last one: {error}");
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
