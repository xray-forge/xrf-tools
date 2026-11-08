use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::effects::distort::distort_parameters::DistortParameters;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::view::view_binding::ViewBinding;

/// Moves the frame where the distortion target says and blurs it by its `z`, at the drawn size before the resolve
/// (`frame/distort.wgsl`).
pub(crate) struct DistortPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
}

impl DistortPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub(crate) fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = DistortParameters::create_layout(source.device);

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
      log::error!("Distortion rejected, drawing with the last one: {error}");
    }
  }

  /// Draws the frame moved by the distortion target into the scene target, within each of `rects`.
  pub(crate) fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &DistortParameters<'_>,
    rects: &[RenderRect],
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);

    for rect in rects {
      pass.set_scissor_rect(rect.x as u32, rect.y as u32, rect.width, rect.height);
      pass.draw(0..3, 0..1);
    }
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
