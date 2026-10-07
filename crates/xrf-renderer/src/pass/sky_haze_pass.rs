use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::sky_haze_parameters::SkyHazeParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::shader::shader_library::ShaderLibrary;

/// Draws the sky as the frame shows it, clouds and all, blurred into a viewport's haze map, which the distance fades
/// into: a few thousand texels a frame, since the skies blend as the clock moves and the clouds drift.
pub struct SkyHazePass {
  layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SkyHazePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, sky_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SkyHazeParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, &layout, sky_layout)?,
      sky_layout: sky_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.layout, &self.sky_layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sky haze rejected, blurring with the last one: {error}"),
      }
    }
  }

  pub fn record(&self, context: &mut RasterContext<'_>, parameters: &SkyHazeParameters, sky: &SkyParameters<'_>) {
    context.bind(parameters);
    context.bind(sky);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/sky_haze",
      "fs_sky_haze",
      &[None, Some(layout), Some(sky_layout)],
      ViewTargets::HAZE,
    )
  }
}
