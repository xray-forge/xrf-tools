use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::effects::distort::distort_parameters::DistortParameters;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Moves the frame where the distortion target says and blurs it by its `z`, at the drawn size before the resolve
/// (`frame/distort.wgsl`).
pub struct DistortPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
}

impl DistortPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = DistortParameters::create_layout(source.device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, view_layout, &layout)?),
      view_layout: view_layout.clone(),
      layout,
    })
  }

  /// Makes its pipeline again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last one where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipeline.refresh(source.cache, || {
      Self::create_pipeline(source, &self.view_layout, &self.layout)
    }) {
      log::error!("Distortion rejected, drawing with the last one: {error}");
    }
  }

  /// Draws the frame moved by the distortion target into the scene target.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &DistortParameters<'_>) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      source,
      "frame/distort",
      "fs_distort",
      &[Some(view_layout), Some(layout)],
      ViewTargets::SCENE,
    )
  }
}
