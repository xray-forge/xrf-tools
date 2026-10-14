use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sky_motion_parameters::SkyMotionParameters;
use crate::pass::view_binding::ViewBinding;

/// Fills the motion target where nothing was drawn with the motion the sky makes as the camera turns, so a resolve
/// reading the target whole carries the sky's history with the view.
pub struct SkyMotionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SkyMotionPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SkyMotionParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(device, source, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sky motion rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Writes the sky's motion into the motion target the pass draws into, where nothing was drawn.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &SkyMotionParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      source,
      "frame/sky_motion",
      "fs_sky_motion",
      &[Some(view_layout), Some(layout)],
      ViewTargets::MOTION,
    )
  }
}
