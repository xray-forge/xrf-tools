use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureAccess, GraphTextureDescriptor, PassParameters, RasterContext, RasterPassBuilder,
  UniformBinding,
};

use crate::frame::view_targets::ViewTargets;
use crate::frame::water_reflection::WaterReflection;
use crate::host::render_bundle::RenderBundle;
use crate::pass::enhanced_water_maps::EnhancedWaterMaps;
use crate::pass::enhanced_water_parameters::EnhancedWaterParameters;
use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_batch_pipelines::WaterBatchPipelines;
use crate::pass::water_blur_pass::WaterBlurPass;
use crate::pass::water_blur_uniform::WaterBlurUniform;
use crate::pass::water_depth_parameters::WaterDepthParameters;
use crate::pass::water_draw::WaterDraw;
use crate::pass::water_reflection_parameters::WaterReflectionParameters;
use crate::pass::water_surface_parameters::WaterSurfaceParameters;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::level_water::LevelWater;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::shader::shader_library::ShaderLibrary;

/// What every water batch binds below its pass's own group: the view, the bindless textures, and the static draws.
type WaterGroups<'a> = (
  &'a ViewBinding,
  &'a wgpu::BindGroup,
  [StaticDrawParameters; StaticLayout::COUNT],
);

/// What the frame's water passes share once declared: the uniforms, both skies, the nearest water, the cull's argument
/// buffers and the groups below the passes' own.
struct WaterFrameHandles<'a> {
  uniform: UniformBinding<WaterUniform>,
  skies: [GraphTexture; 2],
  nearest: GraphTexture,
  args: Vec<GraphBuffer>,
  groups: WaterGroups<'a>,
}

/// Draws a viewport's visible water over its lit scene, tested against the G-buffer's depth without writing it, and the
/// distortion each surface causes into the distortion target while the water distorts. Only the water nearest along
/// each pixel draws: a depth pass writes it first, as the engine's water, drawn in its index order, lets a fold of its
/// surface draw over a nearer one. The enhanced water reads the scene as it stood before the water, which it refracts,
/// and while it reflects draws its reflection first, accumulated over the frames before and blurred.
pub struct WaterPass {
  /// The view's, the bindless textures' and the static draws' layouts, which every water pipeline binds below its own.
  layouts: [wgpu::BindGroupLayout; 3],
  /// One a water batch, in `StaticBatch::list_water` order.
  pipelines: Vec<WaterBatchPipelines>,
  blur: WaterBlurPass,
  maps: EnhancedWaterMaps,
  /// What a binding a pass does not read is given: a texel of nothing.
  nothing: wgpu::TextureView,
  generation: u64,
}

impl WaterPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    shaders: &ShaderLibrary,
    [view_layout, scene_layout, texture_layout]: [&wgpu::BindGroupLayout; 3],
    bundle: &dyn RenderBundle,
  ) -> XrfResult<Self> {
    let layouts: [wgpu::BindGroupLayout; 3] = [view_layout.clone(), texture_layout.clone(), scene_layout.clone()];
    let nothing: wgpu::TextureView = device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("water nothing"),
        size: wgpu::Extent3d {
          width: 1,
          height: 1,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: ViewTargets::SCENE,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default());

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layouts)?,
      blur: WaterBlurPass::new(device, shaders)?,
      maps: EnhancedWaterMaps::new(device, queue, bundle, &nothing),
      nothing,
      layouts,
      generation: shaders.get_generation(),
    })
  }

  /// The bundled maps, which the enhanced rain's puddles and ripples read too.
  pub fn get_maps(&self) -> &EnhancedWaterMaps {
    &self.maps
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Water rejected, drawing with the last one: {error}"),
      }

      if let Err(error) = self.blur.refresh(device, shaders) {
        log::error!("Water blur rejected, blurring with the last one: {error}");
      }
    }
  }

  /// Declares the frame's water: the nearest surface's depth, then the engine's surface, or the enhanced water's scene
  /// copy, reflection and blur and its surface.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    runtime: &mut GraphRuntime,
    draw: WaterDraw<'a>,
  ) {
    let (width, height) = draw.targets.size;
    let frame: WaterFrameHandles<'a> = WaterFrameHandles {
      uniform: runtime.push_uniform(draw.water.get_uniform()),
      skies: [
        bindings.import_view(graph, "sky cube 0", draw.skies[0]),
        bindings.import_view(graph, "sky cube 1", draw.skies[1]),
      ],
      nearest: graph.create_texture(GraphTextureDescriptor::new_2d(
        "water depth",
        width,
        height,
        ViewTargets::DEPTH,
      )),
      args: draw.args.clone(),
      groups: (draw.view, draw.textures, draw.layouts),
    };

    self.add_batch_pass(
      graph
        .add_raster_pass("water depth")
        // Reversed, so nothing is zero and the nearest water is the greatest.
        .depth(GraphDepthAttachment::new(frame.nearest, wgpu::LoadOp::Clear(0.0))),
      WaterDepthParameters { water: frame.uniform },
      (&frame.args, frame.groups),
      |it| &it.depth,
    );

    if draw.water.is_enhanced() {
      self.add_enhanced_passes(graph, bindings, runtime, &draw, &frame);
    } else {
      let parameters: WaterSurfaceParameters<'a> = WaterSurfaceParameters {
        lighting: draw.lighting,
        water: frame.uniform,
        depth_target: draw.targets.depth,
        sky_cube_0: frame.skies[0],
        sky_cube_1: frame.skies[1],
        sky_clamp: draw.sky_sampler,
        nearest_water: frame.nearest,
      };

      self.add_batch_pass(
        Self::begin_surface_pass(graph, &draw),
        parameters,
        (&frame.args, frame.groups),
        |it| &it.engine,
      );
    }
  }

  /// Declares the enhanced water: the scene copied for it to refract, its reflection and the reflection's blur while it
  /// reflects, then its surface.
  fn add_enhanced_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    runtime: &mut GraphRuntime,
    draw: &WaterDraw<'a>,
    frame: &WaterFrameHandles<'a>,
  ) {
    let water: &LevelWater = draw.water;
    let (width, height) = draw.targets.size;
    let enhanced: UniformBinding<EnhancedWaterUniform> = runtime.push_uniform(water.get_enhanced_uniform());
    let mut import = |label: &'static str, view: &'a wgpu::TextureView| bindings.import_view(graph, label, view);
    let nothing: GraphTexture = import("water nothing", &self.nothing);
    let maps: [GraphTexture; 6] = [
      import("water perlin", &self.maps.perlin),
      import("water normal", &self.maps.normal),
      import("water wind", &self.maps.wind),
      import("water caustics", &self.maps.caustics),
      import("water height", &self.maps.height),
      import("water ripples", &self.maps.ripples),
    ];
    let reflection: Option<(&WaterReflection, [GraphTexture; 2], GraphTexture)> =
      water.get_reflection().map(|reflection| {
        (
          reflection,
          [
            import("water reflection 0", &reflection.histories[0]),
            import("water reflection 1", &reflection.histories[1]),
          ],
          import("water blue noise", &self.maps.blue_noise),
        )
      });
    let scene: GraphTexture = Self::add_scene_copy(graph, draw.targets.scene, (width, height));

    let (blurred, clear): (GraphTexture, GraphTexture) = match reflection {
      Some((reflection, histories, blue_noise)) => {
        let index: usize = reflection.index;
        let parameters: WaterReflectionParameters<'a> = WaterReflectionParameters {
          lighting: draw.lighting,
          water: frame.uniform,
          enhanced,
          depth_target: draw.targets.depth,
          sky_cube_0: frame.skies[0],
          sky_cube_1: frame.skies[1],
          sky_clamp: draw.sky_sampler,
          nearest_water: frame.nearest,
          water_scene: scene,
          reflection_history: histories[1 - index],
          blue_noise,
        };

        self.add_batch_pass(
          graph
            .add_raster_pass("water reflection")
            .color(GraphColorAttachment::new(
              histories[index],
              // Transparent where no water draws, so the next frame keeps no history there.
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ))
            .depth(GraphDepthAttachment::new_read_only(draw.targets.depth)),
          parameters,
          (&frame.args, frame.groups),
          |it| &it.reflection,
        );

        let half: (u32, u32) = (width.div_ceil(2), height.div_ceil(2));
        let across: GraphTexture = self.blur.add(
          graph,
          runtime,
          ("water blur across", histories[index], half, WaterBlurUniform::ACROSS),
          enhanced,
        );
        let down: GraphTexture = self.blur.add(
          graph,
          runtime,
          ("water blur down", across, half, WaterBlurUniform::DOWN),
          enhanced,
        );

        (down, histories[index])
      }
      None => (nothing, nothing),
    };

    let parameters: EnhancedWaterParameters<'a> = EnhancedWaterParameters {
      lighting: draw.lighting,
      water: frame.uniform,
      enhanced,
      depth_target: draw.targets.depth,
      sky_cube_0: frame.skies[0],
      sky_cube_1: frame.skies[1],
      sky_clamp: draw.sky_sampler,
      nearest_water: frame.nearest,
      water_scene: scene,
      reflection_blurred: blurred,
      reflection_clear: clear,
      perlin_map: maps[0],
      light_target: draw.targets.light,
      wave_map: maps[1],
      wind_map: maps[2],
      caustics_map: maps[3],
      height_map: maps[4],
      ripple_map: maps[5],
    };

    self.add_batch_pass(
      Self::begin_surface_pass(graph, draw),
      parameters,
      (&frame.args, frame.groups),
      |it| &it.enhanced,
    );
  }

  /// Declares the copy of the scene as it stands before the water, which the enhanced water refracts and reflects.
  fn add_scene_copy(graph: &mut FrameGraph<'_>, source: GraphTexture, (width, height): (u32, u32)) -> GraphTexture {
    let copy: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "water scene",
      width,
      height,
      ViewTargets::SCENE,
    ));

    graph
      .add_encoder_pass("water copy")
      .texture(source, GraphTextureAccess::CopySource)
      .texture(copy, GraphTextureAccess::CopyDestination)
      .record(move |context| {
        let (source, copy) = (context.get_texture(source), context.get_texture(copy));

        context.get_encoder().copy_texture_to_texture(
          source.texture.as_image_copy(),
          copy.texture.as_image_copy(),
          copy.texture.size(),
        );
      });

    copy
  }

  /// Begins the surface's pass: over the scene and the distortion target, against the read-only depth it also samples.
  fn begin_surface_pass<'g, 'a>(graph: &'g mut FrameGraph<'a>, draw: &WaterDraw<'a>) -> RasterPassBuilder<'g, 'a> {
    graph
      .add_raster_pass("water")
      .color(GraphColorAttachment::new(draw.targets.scene, wgpu::LoadOp::Load))
      .color(GraphColorAttachment::new(draw.targets.distortion, wgpu::LoadOp::Load))
      .depth(GraphDepthAttachment::new_read_only(draw.targets.depth))
  }

  /// Adds a pass drawing every water batch from the cull's argument buffers, with its parameters bound and the pipeline
  /// `pick` chooses.
  fn add_batch_pass<'a, P: PassParameters + Send + 'a>(
    &'a self,
    builder: RasterPassBuilder<'_, 'a>,
    parameters: P,
    (args, groups): (&[GraphBuffer], WaterGroups<'a>),
    pick: impl Fn(&WaterBatchPipelines) -> &wgpu::RenderPipeline + Send + 'a,
  ) {
    let args: Vec<GraphBuffer> = args.to_vec();

    args
      .iter()
      .fold(
        StaticDraws::declare_layouts(builder.parameters(&parameters), &groups.2),
        |builder, args| builder.buffer(*args, GraphBufferAccess::Indirect),
      )
      .record(move |context| {
        let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

        context.bind(&parameters);
        self.draw_batches(context, groups, &args, pick);
      });
  }

  /// Draws every water batch each argument buffer lists with the pipeline `pick` chooses, the pass's own group bound.
  fn draw_batches(
    &self,
    context: &mut RasterContext<'_>,
    (view, textures, layouts): WaterGroups<'_>,
    args: &[&wgpu::Buffer],
    pick: impl Fn(&WaterBatchPipelines) -> &wgpu::RenderPipeline,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    for (batch, pipelines) in StaticBatch::list_water().zip(&self.pipelines) {
      context.bind(&layouts[batch.layout.get_index()]);
      context.get_pass().set_pipeline(pick(pipelines));

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }
  }

  /// Every batch's pipelines: the depth and the engine's surface from `static/water`, the enhanced surface from
  /// `static/water_enhanced`, its reflection from `static/water_reflection`, each module with its own group 3.
  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    [view_layout, texture_layout, scene_layout]: &[wgpu::BindGroupLayout; 3],
  ) -> XrfResult<Vec<WaterBatchPipelines>> {
    let engine_module: wgpu::ShaderModule = create_module(device, shaders, "static/water")?;
    let enhanced_module: wgpu::ShaderModule = create_module(device, shaders, "static/water_enhanced")?;
    let reflection_module: wgpu::ShaderModule = create_module(device, shaders, "static/water_reflection")?;
    let create_layout = |label: &str, water: &wgpu::BindGroupLayout| {
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(scene_layout), Some(water)],
        ..Default::default()
      })
    };
    let engine_layout: wgpu::PipelineLayout = create_layout("water", &WaterSurfaceParameters::create_layout(device));
    let enhanced_layout: wgpu::PipelineLayout =
      create_layout("water enhanced", &EnhancedWaterParameters::create_layout(device));
    let depth_layout: wgpu::PipelineLayout = create_layout("water depth", &WaterDepthParameters::create_layout(device));
    let reflection_layout: wgpu::PipelineLayout =
      create_layout("water reflection", &WaterReflectionParameters::create_layout(device));
    let blended = |format: wgpu::TextureFormat| {
      Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      })
    };
    let targets: [Option<wgpu::ColorTargetState>; 2] = [blended(ViewTargets::SCENE), blended(ViewTargets::DISTORTION)];
    let reflection_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(WaterReflection::FORMAT.into())];
    fn vertex(module: &wgpu::ShaderModule) -> wgpu::VertexState<'_> {
      wgpu::VertexState {
        module,
        entry_point: Some("vs_water"),
        compilation_options: Default::default(),
        buffers: &[],
      }
    }
    let primitive = wgpu::PrimitiveState {
      front_face: wgpu::FrontFace::Ccw,
      cull_mode: Some(wgpu::Face::Back),
      ..Default::default()
    };
    let depth_state = |is_written: bool| wgpu::DepthStencilState {
      format: ViewTargets::DEPTH,
      depth_write_enabled: Some(is_written),
      depth_compare: Some(wgpu::CompareFunction::Greater),
      stencil: Default::default(),
      bias: Default::default(),
    };
    let surface = |label: &str,
                   (module, layout, entry): (&wgpu::ShaderModule, &wgpu::PipelineLayout, &str),
                   targets: &[Option<wgpu::ColorTargetState>]| {
      create_checked(device, label, || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some(label),
          layout: Some(layout),
          vertex: vertex(module),
          fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            targets,
          }),
          primitive,
          depth_stencil: Some(depth_state(false)),
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };

    StaticBatch::list_water()
      .map(|_| {
        let depth: wgpu::RenderPipeline = create_checked(device, "water depth", || {
          device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water depth"),
            layout: Some(&depth_layout),
            vertex: vertex(&engine_module),
            fragment: None,
            primitive,
            depth_stencil: Some(depth_state(true)),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
          })
        })?;

        Ok(WaterBatchPipelines {
          depth,
          reflection: surface(
            "water reflection",
            (&reflection_module, &reflection_layout, "fs_water_reflection"),
            &reflection_targets,
          )?,
          engine: surface("water", (&engine_module, &engine_layout, "fs_water"), &targets)?,
          enhanced: surface(
            "water enhanced",
            (&enhanced_module, &enhanced_layout, "fs_water_enhanced"),
            &targets,
          )?,
        })
      })
      .collect()
  }
}
