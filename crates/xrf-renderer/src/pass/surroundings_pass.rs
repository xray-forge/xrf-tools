use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters};

use crate::pass::pipeline_source::PipelineSource;
use crate::pass::surroundings_parameters::SurroundingsParameters;
use crate::pass::view_binding::ViewBinding;

/// Lights one face of the surroundings cube from the G-buffer the static scene drew into it, the sky behind, into the
/// prefiltered cube's first mip, which the mips after it blur.
pub struct SurroundingsPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  generation: u64,
}

impl SurroundingsPass {
  /// Invocations a workgroup runs a side, as `shaders/frame/surroundings.wgsl` declares them.
  pub(crate) const WORKGROUP: u32 = 8;

  pub const CULL_PASS: &'static str = "surroundings cull";
  pub const DRAW_PASS: &'static str = "surroundings g-buffer";
  pub const RESOLVE_PASS: &'static str = "surroundings resolve";

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = SurroundingsParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Surroundings rejected, resolving with the last one: {error}"),
      }
    }
  }

  /// Resolves one face, `size` texels a side, through the face's camera.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    size: u32,
    parameters: &SurroundingsParameters<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_pipeline(&self.pipeline);
    pass.dispatch_workgroups(size.div_ceil(Self::WORKGROUP), size.div_ceil(Self::WORKGROUP), 1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("surroundings", "frame/surroundings", "cs_resolve")
        .with_bind_group_layouts(&[Some(view_layout), Some(layout)]),
    )
  }
}
