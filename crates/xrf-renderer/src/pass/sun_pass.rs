use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Lights a viewport's G-buffer with the sun through its shadow's cascades, into the light its frame accumulates, which
/// it clears first.
pub struct SunPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SunPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SunParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sun rejected, lighting with the last one: {error}"),
      }
    }
  }

  /// Lights the G-buffer by the sun into the light target the pass draws into.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &SunParameters<'_>) {
    context.bind(parameters);

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
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/sun",
      "fs_sun",
      &[Some(view_layout), Some(layout)],
      ViewTargets::LIGHT,
    )
  }
}
