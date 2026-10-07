use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::combine_parameters::CombineParameters;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Lights a viewport's G-buffer with the light it accumulated and the hemisphere, and tonemaps it into its scene.
pub struct CombinePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl CombinePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = CombineParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, &layout, sky_layout)?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.view_layout, &self.layout, &self.sky_layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Combine rejected, combining with the last one: {error}"),
      }
    }
  }

  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &CombineParameters<'_>,
    sky: &SkyParameters<'_>,
  ) {
    context.bind(parameters);
    context.bind(sky);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline_into(
      device,
      shaders,
      "frame/combine",
      "fs_combine",
      &[Some(view_layout), Some(layout), Some(sky_layout)],
      &[Some(ViewTargets::SCENE.into()), Some(ViewTargets::HIGH.into())],
    )
  }
}
