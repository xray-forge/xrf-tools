use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::meter_parameters::MeterParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Reads a spot of the finished scene at a view's centre, and what the G-buffer holds there, into a sample a report
/// reads back.
pub struct MeterPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  generation: u64,
}

impl MeterPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = MeterParameters::create_layout(source.device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipeline again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last one where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Spot meter rejected, metering with the last one: {error}"),
      }
    }
  }

  /// Meters the spot into the sample its parameters write.
  pub fn record(&self, context: &mut ComputeContext<'_>, view: &ViewBinding, parameters: &MeterParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("meter", "frame/meter", "cs_meter")
        .with_bind_group_layouts(&[Some(view_layout), Some(layout)]),
    )
  }
}
