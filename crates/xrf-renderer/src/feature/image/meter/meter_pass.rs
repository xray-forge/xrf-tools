use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters, PipelineSet};

use crate::feature::image::meter::meter_parameters::MeterParameters;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::view::view_binding::ViewBinding;

/// Reads a spot of the finished scene at a view's centre, and what the G-buffer holds there, into a sample a report
/// reads back.
pub(crate) struct MeterPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::ComputePipeline>,
}

impl MeterPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub(crate) fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = MeterParameters::create_layout(source.device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, view_layout, &layout)?),
      view_layout: view_layout.clone(),
      layout,
    })
  }

  /// Makes its pipeline again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last one where a reload is refused.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipeline.refresh(source.cache, || {
      Self::create_pipeline(source, &self.view_layout, &self.layout)
    }) {
      log::error!("Spot meter rejected, metering with the last one: {error}");
    }
  }

  /// Meters the spot into the sample its parameters write.
  pub(crate) fn record(&self, context: &mut ComputeContext<'_>, view: &ViewBinding, parameters: &MeterParameters) {
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
