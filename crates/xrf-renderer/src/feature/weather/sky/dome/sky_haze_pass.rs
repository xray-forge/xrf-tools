use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::weather::sky::dome::sky_haze_parameters::SkyHazeParameters;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;

/// Draws the sky as the frame shows it, clouds and all, blurred into a viewport's haze map, which the distance fades
/// into: a few thousand texels a frame, since the skies blend as the clock moves and the clouds drift.
pub struct SkyHazePass {
  layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
}

impl SkyHazePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, sky_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = SkyHazeParameters::create_layout(device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, &layout, sky_layout)?),
      sky_layout: sky_layout.clone(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipeline.refresh(source.cache, || {
      Self::create_pipeline(source, &self.layout, &self.sky_layout)
    }) {
      log::error!("Sky haze rejected, blurring with the last one: {error}");
    }
  }

  /// Blurs the sky into the haze map the fog's horizon and the sky behind read.
  pub fn record(&self, context: &mut RasterContext<'_>, parameters: &SkyHazeParameters, sky: &SkyParameters<'_>) {
    context.bind(parameters);
    context.bind(sky);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      source,
      "frame/sky_haze",
      "fs_sky_haze",
      &[None, Some(layout), Some(sky_layout)],
      ViewTargets::HAZE,
    )
  }
}
