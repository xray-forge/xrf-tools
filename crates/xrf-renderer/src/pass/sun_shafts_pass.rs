use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::sun_shafts_parameters::SunShaftsParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Adds the sun's light shafts to a viewport's frame after its forward surfaces, as `phase_combine_volumetric` adds
/// what `accum_direct_volumetric` gathered: `one, one`.
pub struct SunShaftsPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SunShaftsPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SunShaftsParameters::create_layout(device);

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
        Err(error) => log::error!("Sun shafts rejected, drawing with the last ones: {error}"),
      }
    }
  }

  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &SunShaftsParameters) {
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
    let added: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };

    let target = |format: wgpu::TextureFormat| {
      Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState {
          color: added,
          alpha: added,
        }),
        write_mask: wgpu::ColorWrites::COLOR,
      })
    };

    create_fullscreen_pipeline_into(
      device,
      shaders,
      "frame/sun_shafts",
      "fs_sun_shafts",
      &[Some(view_layout), Some(layout)],
      &[target(ViewTargets::SCENE), target(ViewTargets::HIGH)],
    )
  }
}
