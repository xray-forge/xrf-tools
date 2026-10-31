use xrf_error::XrfResult;
use xrf_renderer_core::{
  ComputePipelineDescription, FrameGraph, GraphColorAttachment, GraphTexture, GraphTextureDescriptor, PassParameters,
  RasterContext, RenderPipelineDescription,
};

use crate::frame::view_targets::ViewTargets;
use crate::host::render_bundle::RenderBundle;
use crate::pass::enhanced_rain_maps::EnhancedRainMaps;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::rain_blur_parameters::RainBlurParameters;
use crate::pass::rain_counts::RainCounts;
use crate::pass::rain_look_parameters::RainLookParameters;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::rain_rings_reduce_parameters::RainRingsReduceParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::rain_rings::RainRings;

/// Draws the rain over a viewport's finished scene as `RenderLast` does after the forward surfaces: the streaks, then
/// the splashes, blended by their alpha and tested against the G-buffer's depth without writing it. The enhanced
/// streaks draw after the temporal resolve instead, over a copy of the image it shows, which they bend blurred at an
/// eighth of its size, after the frame's distortion, so the water's waves do not bend them. Where they land they draw
/// rings into the rain's rings, which the water and the puddles ripple by.
pub struct RainPass {
  layout: wgpu::BindGroupLayout,
  reduce_layout: wgpu::BindGroupLayout,
  look_layout: wgpu::BindGroupLayout,
  blur_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// The enhanced maps' sampler: anisotropic, so a drop keeps its length's detail however thin it is drawn.
  map_sampler: wgpu::Sampler,
  /// The engine's streaks and splashes and the enhanced splashes, in the scene; the enhanced streaks over the shown
  /// image; and the enhanced drops' rings.
  pipelines: [wgpu::RenderPipeline; 5],
  /// The rings' first level reduced into the rest.
  reduce: wgpu::ComputePipeline,
  /// The image's blocks at an eighth of its size, then their blur, then the image copied whole.
  blur: [wgpu::RenderPipeline; 3],
  maps: EnhancedRainMaps,
  /// A texel covering nothing, bound where the scene's enhanced passes read no blurred image.
  nothing: wgpu::TextureView,
  generation: u64,
}

impl RainPass {
  /// How many times smaller the enhanced drops' blurred image is than the image, as `rt_ssfx_rain` is.
  pub const BLUR_SCALE: u32 = 8;

  /// What the enhanced streaks draw over: the shown image copied, at the scene's own precision.
  pub const LAYER: wgpu::TextureFormat = ViewTargets::SCENE;

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
    let reduce_layout: wgpu::BindGroupLayout = RainRingsReduceParameters::create_layout(device);
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
      pipelines: Self::create_pipelines(source, view_layout, &layout, &look_layout)?,
      blur: Self::create_blur(source, &blur_layout)?,
      reduce: Self::create_reduce(source, &reduce_layout)?,
      maps: EnhancedRainMaps::new(device, queue, bundle, &nothing),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("rain"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
      // `smp_rtlinear` has no mips, so SSS's drops stay short dashes at any distance where trilinear mips smear them
      // along the whole streak; picking the mip along the streak and averaging across it keeps the dashes without
      // their flicker.
      map_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("rain maps"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        anisotropy_clamp: 16,
        ..Default::default()
      }),
      nothing,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
      look_layout,
      blur_layout,
      reduce_layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.view_layout, &self.layout, &self.look_layout).and_then(|pipelines| {
        Ok((
          pipelines,
          Self::create_blur(source, &self.blur_layout)?,
          Self::create_reduce(source, &self.reduce_layout)?,
        ))
      }) {
        Ok((pipelines, blur, reduce)) => {
          self.pipelines = pipelines;
          self.blur = blur;
          self.reduce = reduce;
        }
        Err(error) => log::error!("Rain rejected, falling with the last one: {error}"),
      }
    }
  }

  /// The sampler the streak, the splash and the blurred image are read through, which the parameters bind.
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

  /// A texel covering nothing, for an image a pass binds but does not read.
  pub fn get_nothing(&self) -> &wgpu::TextureView {
    &self.nothing
  }

  /// Declares the enhanced drops' blur of `image`, of `size`, into a transient an eighth of it across: its blocks, then
  /// their blur, which it answers.
  pub fn add_blur<'a>(&'a self, graph: &mut FrameGraph<'a>, image: GraphTexture, size: (u32, u32)) -> GraphTexture {
    let small: (u32, u32) = (size.0.div_ceil(Self::BLUR_SCALE), size.1.div_ceil(Self::BLUR_SCALE));
    let blocks: GraphTexture = self.add_fullscreen(&mut *graph, "rain refraction blocks", &self.blur[0], image, small);

    self.add_fullscreen(&mut *graph, "rain refraction", &self.blur[1], blocks, small)
  }

  /// Declares `image`, of `size`, copied whole into a layer of the enhanced streaks' own, which it answers.
  pub fn add_layer<'a>(&'a self, graph: &mut FrameGraph<'a>, image: GraphTexture, size: (u32, u32)) -> GraphTexture {
    self.add_fullscreen(&mut *graph, "rain layer", &self.blur[2], image, size)
  }

  /// Declares the enhanced drops' rings drawn into the rain's rings' first level, `levels` each of them alone, then
  /// reduced into the rest in one dispatch.
  pub fn add_rings<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    view: &'a ViewBinding,
    parameters: RainParameters<'a>,
    levels: [GraphTexture; RainRings::LEVELS as usize],
    size: u32,
    streaks: u32,
  ) {
    graph
      .add_raster_pass("rain rings")
      .color(GraphColorAttachment::new(
        levels[0],
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .parameters(&parameters)
      .record(move |context| {
        context.bind(&parameters);

        let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

        pass.set_bind_group(0, &view.bind_group, &[]);
        pass.set_pipeline(&self.pipelines[4]);
        pass.draw(0..streaks * 24, 0..1);
      });

    let reduce: RainRingsReduceParameters = RainRingsReduceParameters {
      rings_first: levels[0],
      rings_second: levels[1],
      rings_third: levels[2],
      rings_fourth: levels[3],
      rings_fifth: levels[4],
    };
    let groups: u32 = size / RainRings::BLOCK;

    graph
      .add_compute_pass("rain rings reduce")
      .parameters(&reduce)
      .record(move |context| {
        context.bind(&reduce);

        let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

        pass.set_pipeline(&self.reduce);
        pass.dispatch_workgroups(groups, groups, 1);
      });
  }

  fn add_fullscreen<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    name: &'static str,
    pipeline: &'a wgpu::RenderPipeline,
    source: GraphTexture,
    (width, height): (u32, u32),
  ) -> GraphTexture {
    let target: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(name, width, height, Self::LAYER));
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

  /// Draws in the scene the splashes where the model has indices, and the engine's streaks falling: the engine's, or
  /// with `look` the enhanced splashes, whose streaks draw over the shown image instead.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &RainParameters<'_>,
    look: Option<&RainLookParameters<'_>>,
    counts: RainCounts,
  ) {
    let RainCounts {
      streaks,
      near,
      splash_indices,
    } = counts;
    context.bind(parameters);

    let (streak, splash): (Option<&wgpu::RenderPipeline>, &wgpu::RenderPipeline) = match look {
      Some(look) => {
        context.bind(look);

        (None, &self.pipelines[2])
      }
      None => (Some(&self.pipelines[0]), &self.pipelines[1]),
    };
    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);

    if let Some(streak) = streak {
      pass.set_pipeline(streak);
      pass.draw(0..streaks * 6, 0..1);
    }

    if splash_indices > 0 {
      pass.set_pipeline(splash);
      pass.draw(0..near * splash_indices, 0..1);
    }
  }

  /// Draws the enhanced streaks over the shown image's layer.
  pub fn record_over(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &RainParameters<'_>,
    look: &RainLookParameters<'_>,
    streaks: u32,
  ) {
    context.bind(parameters);
    context.bind(look);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_pipeline(&self.pipelines[3]);
    pass.draw(0..streaks * 6, 0..1);
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    look_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 5]> {
    // `blend(true, srcalpha, invsrcalpha)`, the alpha kept under it.
    let blended = |format: wgpu::TextureFormat| wgpu::ColorTargetState {
      format,
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
    // The distortion under an enhanced streak or splash taken towards its own by its cover, the target's alpha kept.
    let distorting: wgpu::ColorTargetState = wgpu::ColorTargetState {
      format: ViewTargets::DISTORTION,
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
    let engine_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(blended(ViewTargets::SCENE))];
    let splash_targets: [Option<wgpu::ColorTargetState>; 2] = [Some(blended(ViewTargets::SCENE)), Some(distorting)];
    let over_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(blended(Self::LAYER))];
    // Each ring added over the others.
    let ring_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(wgpu::ColorTargetState {
      format: RainRings::FORMAT,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::One,
          dst_factor: wgpu::BlendFactor::One,
          operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent::REPLACE,
      }),
      write_mask: wgpu::ColorWrites::ALL,
    })];
    let engine: [Option<&wgpu::BindGroupLayout>; 2] = [Some(view_layout), Some(layout)];
    let enhanced: [Option<&wgpu::BindGroupLayout>; 3] = [Some(view_layout), Some(layout), Some(look_layout)];
    // Both sides, as the engine's `CULL_NONE`; tested against the scene's depth, where one is attached.
    let create = |(vertex, fragment): (&'static str, &'static str),
                  (layouts, targets): (&[Option<&wgpu::BindGroupLayout>], &[Option<wgpu::ColorTargetState>]),
                  is_depth_tested: bool|
     -> XrfResult<wgpu::RenderPipeline> {
      let description: RenderPipelineDescription = RenderPipelineDescription::new("rain", "frame/rain", vertex)
        .with_fragment(fragment)
        .with_bind_group_layouts(layouts)
        .with_targets(targets.to_vec());

      source.get_render_pipeline(&if is_depth_tested {
        description.with_depth_stencil(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(false),
          depth_compare: Some(wgpu::CompareFunction::Greater),
          stencil: Default::default(),
          bias: Default::default(),
        })
      } else {
        description
      })
    };

    Ok([
      create(("vs_streak", "fs_streak"), (&engine, &engine_targets), true)?,
      create(("vs_splash", "fs_splash"), (&engine, &engine_targets), true)?,
      create(("vs_splash", "fs_splash_enhanced"), (&enhanced, &splash_targets), true)?,
      create(
        ("vs_streak_over", "fs_streak_enhanced"),
        (&enhanced, &over_targets),
        false,
      )?,
      create(("vs_ring", "fs_ring"), (&engine, &ring_targets), false)?,
    ])
  }

  fn create_reduce(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::ComputePipeline> {
    source.get_compute_pipeline(
      &ComputePipelineDescription::new("rain rings reduce", "frame/rain_rings", "cs_reduce_rings")
        .with_bind_group_layouts(&[Some(layout)]),
    )
  }

  fn create_blur(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    let create = |fragment: &'static str| {
      create_fullscreen_pipeline(source, "frame/rain_blur", fragment, &[Some(layout)], Self::LAYER)
    };

    Ok([
      create("fs_rain_block")?,
      create("fs_rain_blur")?,
      create("fs_rain_copy")?,
    ])
  }
}
