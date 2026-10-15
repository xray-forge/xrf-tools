use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::ray_guides_parameters::RayGuidesParameters;
use crate::pass::view_binding::ViewBinding;

/// Works out the guides DLSS ray reconstruction reads beside the frame, at the frame's size: each surface's specular
/// albedo, its world normal with its roughness, and how far its reflection ray and its indirect light ray went.
pub struct RayGuidesPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl RayGuidesPass {
  /// The share of the light around it each surface reflects.
  pub const SPECULAR_ALBEDO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  /// The world normal, and the linear roughness in the fourth channel.
  pub const NORMALS: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Metres the reflection ray went, and the indirect light's.
  pub const HIT_DISTANCE: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
  /// How far a pixel is reconstructed from this frame alone.
  pub const BIAS: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = RayGuidesParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(device, source, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Ray guides rejected, drawing with the last one: {error}"),
      }
    }
  }

  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &RayGuidesParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline_into(
      device,
      source,
      "frame/ray_guides",
      "fs_ray_guides",
      &[Some(view_layout), Some(layout)],
      &[
        Some(Self::SPECULAR_ALBEDO.into()),
        Some(Self::NORMALS.into()),
        Some(Self::HIT_DISTANCE.into()),
        Some(Self::BIAS.into()),
        Some(Self::HIT_DISTANCE.into()),
      ],
    )
  }
}
