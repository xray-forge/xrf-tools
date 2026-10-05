use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::upscale_targets::UpscaleTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::shader::shader_library::ShaderLibrary;

/// FSR 1 over a frame drawn smaller than its viewport: EASU upscales it, RCAS sharpens the upscaled frame.
pub struct UpscalePass {
  layout: wgpu::BindGroupLayout,
  /// EASU's, then RCAS's.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl UpscalePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = UpscaleParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layout)?,
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Upscaling rejected, upscaling with the last one: {error}"),
      }
    }
  }

  /// Draws EASU's upscale (`stage` 0) or RCAS's sharpening (1) into the target the pass draws into.
  pub fn record(&self, context: &mut RasterContext<'_>, stage: usize, parameters: &UpscaleParameters) {
    context.get_pass().set_pipeline(&self.pipelines[stage]);
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let create = |fragment: &str| {
      create_fullscreen_pipeline(
        device,
        shaders,
        "frame/upscale",
        fragment,
        &[Some(layout)],
        UpscaleTargets::FORMAT,
      )
    };

    Ok([create("fs_easu")?, create("fs_rcas")?])
  }
}
