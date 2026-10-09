use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::upscale_parameters::UpscaleParameters;

/// FSR 1 over a frame drawn smaller than its viewport: EASU upscales it, RCAS sharpens the upscaled frame.
pub struct UpscalePass {
  layout: wgpu::BindGroupLayout,
  /// EASU's, then RCAS's.
  pipelines: [wgpu::RenderPipeline; 3],
  generation: u64,
}

impl UpscalePass {
  /// The upscaled and the sharpened frame's format.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, source: PipelineSource<'_>) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = UpscaleParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, source, &layout)?,
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(device, source, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Upscaling rejected, upscaling with the last one: {error}"),
      }
    }
  }

  /// EASU's upscale, of a frame smaller than the viewport.
  pub const EASU: usize = 0;
  /// RCAS's sharpening of an upscaled frame.
  pub const RCAS: usize = 1;
  /// The area-averaged downscale of a frame larger than the viewport.
  pub const DOWNSAMPLE: usize = 2;

  /// Draws a stage (`EASU`, `RCAS` or `DOWNSAMPLE`) into the target the pass draws into.
  pub fn record(&self, context: &mut RasterContext<'_>, stage: usize, parameters: &UpscaleParameters) {
    context.get_pass().set_pipeline(&self.pipelines[stage]);
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    let create = |fragment: &str| {
      create_fullscreen_pipeline(device, source, "frame/upscale", fragment, &[Some(layout)], Self::FORMAT)
    };

    Ok([create("fs_easu")?, create("fs_rcas")?, create("fs_downsample")?])
  }
}
