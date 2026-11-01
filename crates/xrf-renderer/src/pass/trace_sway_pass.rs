use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters, PipelineSet};

use crate::pass::compute_grid::ComputeGrid;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::trace_sway_parameters::TraceSwayParameters;

/// Sways the vertices of the trees a view's trace stands swayed (`static/trace_sway`), placed in the world, for their
/// acceleration structures to be built over.
pub struct TraceSwayPass {
  layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::ComputePipeline>,
  grid: ComputeGrid,
}

impl TraceSwayPass {
  /// Invocations a workgroup runs, as the source declares it.
  pub(crate) const WORKGROUP: u32 = 64;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = TraceSwayParameters::create_layout(device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, &layout)?),
      grid: ComputeGrid::new(device),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self
      .pipeline
      .refresh(source.cache, || Self::create_pipeline(source, &self.layout))
    {
      log::error!("Trace sway rejected, swaying with the last one: {error}");
    }
  }

  /// Sways `vertices` vertices of the instances the parameters name.
  pub fn record(&self, context: &mut ComputeContext<'_>, parameters: &TraceSwayParameters, vertices: u32) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();
    let (x, y) = self.grid.get_size(vertices.div_ceil(Self::WORKGROUP));

    pass.set_pipeline(&self.pipeline);
    pass.dispatch_workgroups(x, y, 1);
  }

  fn create_pipeline(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("trace sway", "static/trace_sway", "cs_trace_sway")
        .with_bind_group_layouts(&[Some(layout)]),
    )
  }
}
