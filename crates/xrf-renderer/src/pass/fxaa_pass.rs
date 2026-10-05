use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::fxaa_parameters::FxaaParameters;
use crate::shader::shader_library::ShaderLibrary;

/// FXAA over a viewport's scene as drawn, into a frame's target the scene is copied back from.
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
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = FxaaParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fxaa"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("FXAA rejected, smoothing with the last one: {error}"),
      }
    }
  }

  /// The sampler it reads the scene with, which its parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  pub fn record(&self, context: &mut RasterContext<'_>, parameters: &FxaaParameters<'_>) {
    context.get_pass().set_pipeline(&self.pipeline);
    context.bind(parameters);
    context.get_pass().draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/fxaa",
      "fs_fxaa",
      &[Some(layout)],
      ViewTargets::SCENE,
    )
  }
}
