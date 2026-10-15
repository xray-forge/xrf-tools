use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::fxaa_parameters::FxaaParameters;
use crate::pass::pipeline_source::PipelineSource;

/// FXAA over a viewport's scene as drawn, into a target of its own, which the frame goes on from.
pub struct FxaaPass {
  layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl FxaaPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = FxaaParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fxaa"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("FXAA rejected, smoothing with the last one: {error}"),
      }
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
