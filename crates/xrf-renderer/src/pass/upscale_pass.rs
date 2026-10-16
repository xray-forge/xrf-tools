use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::pass::upscale_stage::UpscaleStage;

/// FSR 1 over a frame drawn smaller than its viewport: EASU upscales it, RCAS sharpens the upscaled frame.
pub struct UpscalePass {
  layout: wgpu::BindGroupLayout,
  /// One a stage, at its index.
  pipelines: [wgpu::RenderPipeline; UpscaleStage::COUNT],
  generation: u64,
}

impl UpscalePass {
  /// The upscaled and the sharpened frame's format.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = UpscaleParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, &layout)?,
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Upscaling rejected, upscaling with the last one: {error}"),
      }
    }
  }

  /// Draws a stage into the target the pass draws into.
  pub fn record(&self, context: &mut RasterContext<'_>, stage: UpscaleStage, parameters: &UpscaleParameters) {
    context.get_pass().set_pipeline(&self.pipelines[stage.get_index()]);
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; UpscaleStage::COUNT]> {
    let [easu, rcas, downsample] = UpscaleStage::ALL.map(|stage| {
      create_fullscreen_pipeline(
        source,
        "frame/upscale",
        stage.get_entry(),
        &[Some(layout)],
        Self::FORMAT,
      )
    });

    Ok([easu?, rcas?, downsample?])
  }
}
