use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphTexture, GraphTextureDescriptor, PassParameters, RasterContext,
  RenderPipelineDescription,
};

use crate::frame::view_targets::ViewTargets;
use crate::host::render_bundle::RenderBundle;
use crate::pass::enhanced_rain_maps::EnhancedRainMaps;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::rain_blur_parameters::RainBlurParameters;
use crate::pass::rain_look_parameters::RainLookParameters;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::view_binding::ViewBinding;

/// Draws the rain over a viewport's finished scene as `RenderLast` does after the forward surfaces: the streaks, then
/// the splashes, blended by their alpha and tested against the G-buffer's depth without writing it; for the enhanced
/// drops, the scene blurred at an eighth of its size first, which they bend.
pub struct RainPass {
  layout: wgpu::BindGroupLayout,
  look_layout: wgpu::BindGroupLayout,
  blur_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// The enhanced maps' sampler: anisotropic, so a drop keeps its length's detail however thin it is drawn.
  map_sampler: wgpu::Sampler,
  /// The engine's streaks and splashes, then the enhanced ones.
  pipelines: [wgpu::RenderPipeline; 4],
  /// The scene's blocks at an eighth of its size, then their blur.
  blur: [wgpu::RenderPipeline; 2],
  maps: EnhancedRainMaps,
  generation: u64,
}

impl RainPass {
  /// How many times smaller the enhanced drops' blurred scene is than the scene, as `rt_ssfx_rain` is.
  pub const BLUR_SCALE: u32 = 8;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    queue: &wgpu::Queue,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    bundle: &dyn RenderBundle,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = RainParameters::create_layout(device);
    let look_layout: wgpu::BindGroupLayout = RainLookParameters::create_layout(device);
    let blur_layout: wgpu::BindGroupLayout = RainBlurParameters::create_layout(device);
    // A texel covering nothing, for a map the bundle cannot give.
    let nothing: wgpu::TextureView = device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("rain nothing"),
        size: wgpu::Extent3d {
          width: 1,
          height: 1,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default());

    Ok(Self {
      pipelines: Self::create_pipelines(source, view_layout, (&layout, &look_layout))?,
      blur: Self::create_blur(source, &blur_layout)?,
      maps: EnhancedRainMaps::new(device, queue, bundle, &nothing),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("rain"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
      // `smp_rtlinear` has no mips, so SSS's drops stay two short dashes a streak at any distance where trilinear
      // mips smear them along the whole streak; picking the mip along the streak and averaging across it keeps the
      // dashes without their flicker.
      map_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("rain maps"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: 16,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
      look_layout,
      blur_layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, (&self.layout, &self.look_layout))
        .and_then(|pipelines| Ok((pipelines, Self::create_blur(source, &self.blur_layout)?)))
      {
        Ok((pipelines, blur)) => {
          self.pipelines = pipelines;
          self.blur = blur;
        }
        Err(error) => log::error!("Rain rejected, falling with the last one: {error}"),
      }
    }
  }

  /// The sampler the streak, the splash and the blurred scene are read through, which the parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// The enhanced drops' and splash's maps.
  pub fn get_maps(&self) -> &EnhancedRainMaps {
    &self.maps
  }

  /// The sampler the enhanced maps are read through.
  pub fn get_map_sampler(&self) -> &wgpu::Sampler {
    &self.map_sampler
  }

  /// Declares the enhanced drops' blur of `scene`, of the scene's `size`, into a transient an eighth of it across: its
  /// blocks, then their blur, which it answers.
  pub fn add_blur<'a>(&'a self, graph: &mut FrameGraph<'a>, scene: GraphTexture, size: (u32, u32)) -> GraphTexture {
    let small: (u32, u32) = (size.0.div_ceil(Self::BLUR_SCALE), size.1.div_ceil(Self::BLUR_SCALE));
    let blocks: GraphTexture =
      self.add_blur_stage(&mut *graph, ("rain refraction blocks", &self.blur[0]), scene, small);

    self.add_blur_stage(&mut *graph, ("rain refraction", &self.blur[1]), blocks, small)
  }

  fn add_blur_stage<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    (name, pipeline): (&'static str, &'a wgpu::RenderPipeline),
    source: GraphTexture,
    (width, height): (u32, u32),
  ) -> GraphTexture {
    let target: GraphTexture =
      graph.create_texture(GraphTextureDescriptor::new_2d(name, width, height, ViewTargets::SCENE));
    let parameters: RainBlurParameters<'a> = RainBlurParameters {
      source,
      source_sampler: &self.sampler,
    };

    graph
      .add_raster_pass(name)
      .color(GraphColorAttachment::new(
        target,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .parameters(&parameters)
      .record(move |context| {
        context.bind(&parameters);
        context.get_pass().set_pipeline(pipeline);
        context.get_pass().draw(0..3, 0..1);
      });

    target
  }

  /// Draws the streaks falling, and the splashes where the model has indices: the enhanced ones where `look` is given.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    (parameters, look): (&RainParameters<'_>, Option<&RainLookParameters<'_>>),
    (streaks, splash_indices): (u32, u32),
  ) {
    context.bind(parameters);

    let pipelines: &[wgpu::RenderPipeline] = match look {
      Some(look) => {
        context.bind(look);

        &self.pipelines[2..]
      }
      None => &self.pipelines[..2],
    };
    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_pipeline(&pipelines[0]);
    pass.draw(0..streaks * 6, 0..1);

    if splash_indices > 0 {
      pass.set_pipeline(&pipelines[1]);
      pass.draw(0..streaks * splash_indices, 0..1);
    }
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    (layout, look_layout): (&wgpu::BindGroupLayout, &wgpu::BindGroupLayout),
  ) -> XrfResult<[wgpu::RenderPipeline; 4]> {
    // `blend(true, srcalpha, invsrcalpha)`, the alpha kept under it.
    let target: wgpu::ColorTargetState = wgpu::ColorTargetState {
      format: ViewTargets::SCENE,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::SrcAlpha,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::One,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
      }),
      write_mask: wgpu::ColorWrites::ALL,
    };
    // Each enhanced drop takes the distortion under it towards none and the motion under it towards its own fall, by
    // the cover each output's alpha gives, the distortion's own alpha kept.
    let covering = |format: wgpu::TextureFormat| wgpu::ColorTargetState {
      format,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::SrcAlpha,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::Zero,
          dst_factor: wgpu::BlendFactor::One,
          operation: wgpu::BlendOperation::Add,
        },
      }),
      write_mask: wgpu::ColorWrites::ALL,
    };
    let engine_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(target.clone())];
    let enhanced_targets: [Option<wgpu::ColorTargetState>; 3] = [
      Some(target),
      Some(covering(ViewTargets::DISTORTION)),
      Some(covering(ViewTargets::MOTION)),
    ];
    let engine: [Option<&wgpu::BindGroupLayout>; 2] = [Some(view_layout), Some(layout)];
    let enhanced: [Option<&wgpu::BindGroupLayout>; 3] = [Some(view_layout), Some(layout), Some(look_layout)];
    // Both sides, as the engine's `CULL_NONE`.
    let create = |(vertex, fragment): (&'static str, &'static str),
                  (layouts, targets): (&[Option<&wgpu::BindGroupLayout>], &[Option<wgpu::ColorTargetState>])|
     -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &RenderPipelineDescription::new("rain", "frame/rain", vertex)
          .with_fragment(fragment)
          .with_bind_group_layouts(layouts)
          .with_depth_stencil(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::Greater),
            stencil: Default::default(),
            bias: Default::default(),
          })
          .with_targets(targets.to_vec()),
      )
    };

    Ok([
      create(("vs_streak", "fs_streak"), (&engine, &engine_targets))?,
      create(("vs_splash", "fs_splash"), (&engine, &engine_targets))?,
      create(("vs_streak", "fs_streak_enhanced"), (&enhanced, &enhanced_targets))?,
      create(("vs_splash", "fs_splash_enhanced"), (&enhanced, &enhanced_targets))?,
    ])
  }

  fn create_blur(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let create = |fragment: &'static str| {
      create_fullscreen_pipeline(source, "frame/rain_blur", fragment, &[Some(layout)], ViewTargets::SCENE)
    };

    Ok([create("fs_rain_block")?, create("fs_rain_blur")?])
  }
}
