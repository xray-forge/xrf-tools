use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::contact_shadow_parameters::ContactShadowParameters;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Marches each drawn pixel's ray towards the sun over the frame's depth, into how much sunlight reaches it.
pub struct ContactShadowPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  /// What the sun reads while no contact shadow is drawn: one texel, all lit.
  lit: wgpu::TextureView,
  generation: u64,
}

impl ContactShadowPass {
  /// The sunlight each pixel keeps, from none to all.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = ContactShadowParameters::create_layout(device);
    let lit: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("contact shadows lit"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: Self::FORMAT,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &[u8::MAX],
      )
      .create_view(&Default::default());

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      lit,
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Contact shadows rejected, marching with the last ones: {error}"),
      }
    }
  }

  /// The texel the sun reads in place of contact shadows not drawn.
  pub fn get_lit(&self) -> &wgpu::TextureView {
    &self.lit
  }

  /// Marches the frame's pixels into the target the pass draws into.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &ContactShadowParameters) {
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
      "frame/contact_shadows",
      "fs_contact_shadows",
      &[Some(view_layout), Some(layout)],
      Self::FORMAT,
    )
  }
}
