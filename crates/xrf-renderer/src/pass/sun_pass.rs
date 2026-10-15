use wgpu::util::DeviceExt;
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::contract::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_with;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::sun_trace_pass::SunTracePass;
use crate::pass::view_binding::ViewBinding;

/// Lights a viewport's G-buffer with the sun through its shadow's cascades, or what its traced shadow saw, into the
/// light its frame accumulates, which it clears first, and writes beside it how much of the sun reaches each pixel.
pub struct SunPass {
  layout: wgpu::BindGroupLayout,
  /// A texel a layer of the sun's maps, all at the far plane, which the frame binds while no cascade is drawn.
  unshadowed: wgpu::TextureView,
  /// A lit texel, which the frame binds where the sun's shadow is not traced.
  untraced: wgpu::TextureView,
  view_layout: wgpu::BindGroupLayout,
  /// Shadowed by the cascades, then by the traced shadow, for each width the cascades' filter reaches (`MAX_REACH`
  /// apart), so its loops compile to their count.
  pipelines: [[wgpu::RenderPipeline; 2]; Self::REACHES],
  generation: u64,
}

impl SunPass {
  /// How much of the sun reaches a pixel, past its shadow, its contact shadow and its relief.
  pub const SEEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

  /// The filter's widths there are pipelines for: none to three texels each way, as the settings offer.
  const REACHES: usize = 4;

  /// The sun's maps with nothing in them, which the frame binds while no cascade is drawn.
  pub fn get_unshadowed(&self) -> &wgpu::TextureView {
    &self.unshadowed
  }

  /// A lit texel, which the frame binds where the sun's shadow is not traced.
  pub fn get_untraced(&self) -> &wgpu::TextureView {
    &self.untraced
  }

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SunParameters::create_layout(device);

    Ok(Self {
      untraced: device
        .create_texture_with_data(
          queue,
          &wgpu::TextureDescriptor {
            label: Some("sun untraced"),
            size: wgpu::Extent3d {
              width: 1,
              height: 1,
              depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SunTracePass::FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
          },
          Default::default(),
          &[u8::MAX],
        )
        .create_view(&Default::default()),
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
      pipelines: Self::create_pipelines(source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Sun rejected, lighting with the last one: {error}"),
      }
    }
  }

  /// Lights the G-buffer by the sun into the light target the pass draws into, shadowed by its traced shadow where
  /// `is_traced`, its cascades filtered `filter_reach` texels each way.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &SunParameters<'_>,
    (is_traced, filter_reach): (bool, u32),
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[(filter_reach as usize).min(Self::REACHES - 1)][usize::from(is_traced)]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[[wgpu::RenderPipeline; 2]; Self::REACHES]> {
    let create = |entry: &'static str, reach: usize| {
      create_fullscreen_pipeline_with(
        source,
        ("frame/sun", entry, &[("SUN_FILTER_REACH", reach as f64)]),
        &[Some(view_layout), Some(layout)],
        &[Some(ViewTargets::LIGHT.into()), Some(Self::SEEN_FORMAT.into())],
      )
    };
    let pair = |reach: usize| -> XrfResult<[wgpu::RenderPipeline; 2]> {
      Ok([create("fs_sun", reach)?, create("fs_sun_traced", reach)?])
    };

    Ok([pair(0)?, pair(1)?, pair(2)?, pair(3)?])
  }
}
