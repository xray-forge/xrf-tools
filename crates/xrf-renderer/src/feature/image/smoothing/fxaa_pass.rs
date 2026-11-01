use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::image::smoothing::fxaa_parameters::FxaaParameters;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;

/// FXAA over a viewport's scene as drawn, into a target of its own, which the frame goes on from.
pub struct FxaaPass {
  layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
}

impl FxaaPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = FxaaParameters::create_layout(device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, &layout)?),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fxaa"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
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
      log::error!("FXAA rejected, smoothing with the last one: {error}");
    }
  }

  /// The sampler it reads the scene with, which its parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Smooths the frame's edges into the smoothed target.
  pub fn record(&self, context: &mut RasterContext<'_>, parameters: &FxaaParameters<'_>) {
    context.get_pass().set_pipeline(&self.pipeline);
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipeline(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(source, "frame/fxaa", "fs_fxaa", &[Some(layout)], ViewTargets::SCENE)
  }
}
