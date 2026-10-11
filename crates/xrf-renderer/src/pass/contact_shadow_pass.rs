use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::contact_shadow_parameters::ContactShadowParameters;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Marches a ray towards the sun over the frame's depth from one drawn pixel of each two by two, into how much sunlight
/// reaches it and how far along the view it stands, which the sun brings up to every pixel by depth.
pub struct ContactShadowPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  /// What the sun reads while no contact shadow is drawn: one texel, all lit.
  lit: wgpu::TextureView,
  generation: u64,
}

impl ContactShadowPass {
  /// The sunlight each marched pixel keeps, from none to all, then its distance along the view.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

  /// The stand-in's texel: all lit, as far as half floats reach.
  const LIT: [u16; 2] = [0x3c00, 0x7bff];

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(queue: &wgpu::Queue, source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

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
        bytemuck::cast_slice(&Self::LIT),
      )
      .create_view(&Default::default());

    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      lit,
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.view_layout, &self.layout) {
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
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      source,
      "frame/contact_shadows",
      "fs_contact_shadows",
      &[Some(view_layout), Some(layout)],
      Self::FORMAT,
    )
  }
}
