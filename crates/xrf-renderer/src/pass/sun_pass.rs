use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::contract::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::view_binding::ViewBinding;

/// Lights a viewport's G-buffer with the sun through its shadow's cascades, into the light its frame accumulates, which
/// it clears first.
pub struct SunPass {
  layout: wgpu::BindGroupLayout,
  /// A texel a layer of the sun's maps, all at the far plane, which the frame binds while no cascade is drawn.
  unshadowed: wgpu::TextureView,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl SunPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  /// The sun's maps with nothing in them, which the frame binds while no cascade is drawn.
  pub fn get_unshadowed(&self) -> &wgpu::TextureView {
    &self.unshadowed
  }

  pub fn new(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SunParameters::create_layout(device);

    Ok(Self {
      unshadowed: device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some("sun unshadowed"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: RENDER_MAX_SHADOW_CASCADES as u32,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: SunShadowMaps::FORMAT,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor {
          dimension: Some(wgpu::TextureViewDimension::D2Array),
          ..Default::default()
        }),
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
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      source,
      "frame/sun",
      "fs_sun",
      &[Some(view_layout), Some(layout)],
      ViewTargets::LIGHT,
    )
  }
}
