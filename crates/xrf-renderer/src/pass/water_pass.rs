use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureDescriptor, PassParameters, RasterContext, RasterPassBuilder, RenderPipelineDescription,
  StorageValue, UniformBinding,
};

use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_bundle::RenderBundle;
use crate::pass::enhanced_water_maps::EnhancedWaterMaps;
use crate::pass::enhanced_water_parameters::EnhancedWaterParameters;
use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::reflection_scene_parameters::ReflectionSceneParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::trace_request_handles::TraceRequestHandles;
use crate::pass::trace_request_parameters::TraceRequestParameters;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_blur_pass::WaterBlurPass;
use crate::pass::water_blur_uniform::WaterBlurUniform;
use crate::pass::water_depth_parameters::WaterDepthParameters;
use crate::pass::water_draw::WaterDraw;
use crate::pass::water_parameter_layouts::WaterParameterLayouts;
use crate::pass::water_pipelines::WaterPipelines;
use crate::pass::water_reflection_parameters::WaterReflectionParameters;
use crate::pass::water_surface_parameters::WaterSurfaceParameters;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::feature::water_feature::WaterFeature;
use crate::scene::level::feature::water_reflection::WaterReflection;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;

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

/// Draws a viewport's visible water over its lit scene, tested against the G-buffer's depth without writing it, the
/// distortion each surface causes into the distortion target while the water distorts, and its motion over what lies
/// under it. Only the water nearest along
/// each pixel draws: a depth pass writes it first, as the engine's water, drawn in its index order, lets a fold of its
/// surface draw over a nearer one. The enhanced water reads the scene as it stood before the water, which it refracts,
/// and while it reflects draws its reflection first, accumulated over the frames before and blurred.
pub struct WaterPass {
  /// The view's, the bindless textures' and the static draws' layouts, which every water pipeline binds below its own.
  layouts: [wgpu::BindGroupLayout; 3],
  /// The list the reflection leaves the rays the screen does not answer in, bound at group 3.
  request_layout: wgpu::BindGroupLayout,
  parameter_layouts: WaterParameterLayouts,
  /// What every batch draws with, its own static draws' group bound before it.
  pipelines: WaterPipelines,
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
    queue: &wgpu::Queue,
    source: PipelineSource<'_>,
    [view_layout, scene_layout, texture_layout]: [&wgpu::BindGroupLayout; 3],
    bundle: &dyn RenderBundle,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layouts: [wgpu::BindGroupLayout; 3] = [view_layout.clone(), texture_layout.clone(), scene_layout.clone()];
    let request_layout: wgpu::BindGroupLayout = TraceRequestParameters::create_layout(device);
    let parameter_layouts: WaterParameterLayouts = WaterParameterLayouts::new(device);
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
      pipelines: Self::create_pipelines(source, &layouts, &request_layout, &parameter_layouts)?,
      blur: WaterBlurPass::new(source)?,
      maps: EnhancedWaterMaps::new(device, queue, bundle, &nothing),
      nothing,
      layouts,
      request_layout,
      parameter_layouts,
      generation: source.get_generation(),
    })
  }

  /// Draws the water a view's list holds into the depth the pass draws into, its waves lifted as the frame's: an
  /// overhead map's view, its own lists and arguments.
  pub fn record_depth(
    &self,
    context: &mut RasterContext<'_>,
    groups: (
      &ViewBinding,
      &wgpu::BindGroup,
      [StaticDrawParameters; StaticLayout::COUNT],
    ),
    args: &wgpu::Buffer,
    parameters: &WaterDepthParameters,
  ) {
    context.bind(parameters);
    self.draw_batches(context, groups, &[args], |it| &it.depth);
  }

  /// The bundled maps, which the enhanced rain's puddles and ripples read too.
  pub fn get_maps(&self) -> &EnhancedWaterMaps {
    &self.maps
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(source, &self.layouts, &self.request_layout, &self.parameter_layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Water rejected, drawing with the last one: {error}"),
      }

      if let Err(error) = self.blur.refresh(source) {
        log::error!("Water blur rejected, blurring with the last one: {error}");
      }
    }
  }

  /// Declares the frame's water: the nearest surface's depth, then the engine's surface, or the enhanced water's scene
  /// copy, reflection and blur and its surface. Answers the G-buffer's depth with the nearest water's over it, which
  /// the resolve reprojects the water's pixels by.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    runtime: &mut GraphRuntime,
    draw: WaterDraw<'a>,
  ) -> GraphTexture {
    let (width, height) = draw.targets.size;
    let frame: WaterFrameHandles<'a> = WaterFrameHandles {
      uniform: runtime.push_uniform(draw.water.get_uniform()),
      skies: [
        bindings.import_view(graph, "sky cube 0", draw.skies[0]),
        bindings.import_view(graph, "sky cube 1", draw.skies[1]),
      ],
      nearest: graph.create_texture(GraphTextureDescriptor::new_2d(
        "depth with water",
        width,
        height,
        ViewTargets::DEPTH,
      )),
      args: draw.args.clone(),
      groups: (draw.view, draw.textures, draw.layouts),
    };

    add_texture_copy(graph, "water depth copy", draw.targets.depth, frame.nearest);
    self.add_batch_pass(
      graph
        .add_raster_pass("water depth")
        // Reversed, so the nearest water is the greatest; over the G-buffer's, where water behind it stays unwritten.
        .depth(GraphDepthAttachment::new(frame.nearest, wgpu::LoadOp::Load)),
      (WaterDepthParameters { water: frame.uniform }, None),
      (&frame.args, frame.groups),
      |it| &it.depth,
    );

    if draw.water.is_enhanced() {
      self.add_enhanced_passes(graph, bindings, runtime, &draw, &frame);
    } else {
      let parameters: WaterSurfaceParameters<'a> = WaterSurfaceParameters {
        lighting: draw.lit.lighting,
        water: frame.uniform,
        depth_target: draw.targets.depth,
        sky_cube_0: frame.skies[0],
        sky_cube_1: frame.skies[1],
        sky_clamp: draw.sky_sampler,
        nearest_water: frame.nearest,
      };

      self.add_batch_pass(
        Self::begin_surface_pass(graph, &draw),
        (parameters, None),
        (&frame.args, frame.groups),
        |it| &it.engine,
      );
    }

    frame.nearest
  }

  /// Declares the enhanced water over the frame's copy before it, which it refracts: its reflection and the
  /// reflection's blur while it reflects, then its surface; nothing without the copy.
  fn add_enhanced_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    runtime: &mut GraphRuntime,
    draw: &WaterDraw<'a>,
    frame: &WaterFrameHandles<'a>,
  ) {
    let water: &WaterFeature = draw.water;
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
          reflection
            .histories
            .get_sides()
            .map(|side| import("water reflection", &side.view)),
          import("water blue noise", &self.maps.blue_noise),
        )
      });
    let Some(scene) = draw.before_water else {
      return;
    };

    let (blurred, clear): (GraphTexture, GraphTexture) = match reflection {
      Some(reflection) => self.add_reflection_passes(
        (graph, bindings, runtime),
        (draw, frame),
        (enhanced, scene, nothing),
        reflection,
      ),
      None => (nothing, nothing),
    };

    let parameters: EnhancedWaterParameters<'a> = EnhancedWaterParameters {
      lighting: draw.lit.lighting,
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
      (parameters, None),
      (&frame.args, frame.groups),
      |it| &it.enhanced,
    );
  }

  /// Declares the water's reflection over the frame's copy before it (`scene`): traced on the screen into the current
  /// reflection, the rays the screen left traced against the level's geometry, resolved over time into its history and
  /// blurred. Answers the reflection blurred and the history written.
  fn add_reflection_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    (draw, frame): (&WaterDraw<'a>, &WaterFrameHandles<'a>),
    (enhanced, scene, nothing): (UniformBinding<EnhancedWaterUniform>, GraphTexture, GraphTexture),
    (reflection, histories, blue_noise): (&'a WaterReflection, [GraphTexture; 2], GraphTexture),
  ) -> (GraphTexture, GraphTexture) {
    let (width, height) = draw.targets.size;
    let [written, read]: [GraphTexture; 2] = histories;
    let current: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "water reflection current",
      width,
      height,
      WaterReflection::CURRENT_FORMAT,
    ));
    let listed: TraceRequestHandles = draw.scene_trace.add_reset(
      graph,
      bindings,
      ("water reflection requests clear", &reflection.requests),
    );
    let parameters: WaterReflectionParameters<'a> = WaterReflectionParameters {
      lighting: draw.lit.lighting,
      water: frame.uniform,
      enhanced,
      depth_target: draw.targets.depth,
      sky_cube_0: frame.skies[0],
      sky_cube_1: frame.skies[1],
      sky_clamp: draw.sky_sampler,
      nearest_water: frame.nearest,
      water_scene: scene,
      reflection_history: read,
      blue_noise,
      water_current: nothing,
    };

    self.add_batch_pass(
      graph
        .add_raster_pass("water reflection")
        .color(GraphColorAttachment::new(
          current,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .depth(GraphDepthAttachment::new_read_only(draw.targets.depth)),
      (parameters, Some(listed.to_listed())),
      (&frame.args, frame.groups),
      |it| &it.reflection,
    );

    // The rays the screen left, traced against the level's geometry, their hits written into the reflection.
    if let Some((sky, trace, textures)) = draw.traced {
      let (trace_requests, trace_request_head) = listed.to_read();
      let no_ends: GraphTexture = bindings.import_view(graph, "scene trace no ends", draw.scene_trace.get_no_ends());

      draw.scene_trace.add_trace(
        graph,
        (("water scene dispatch", "water scene"), listed),
        draw.view,
        (
          ReflectionSceneParameters {
            lighting: draw.lit.lighting,
            material_lut: draw.lit.material_lut,
            lut_sampler: draw.lit.lut_sampler,
            exposure: StorageValue::new(draw.lit.exposure),
            trace_requests,
            trace_request_head,
            traced_out: current,
            ends_out: no_ends,
          },
          sky,
        ),
        (trace, textures),
      );
    }

    self.add_batch_pass(
      graph
        .add_raster_pass("water reflection resolve")
        .color(GraphColorAttachment::new(
          written,
          // Transparent where no water draws, so the next frame keeps no history there.
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .depth(GraphDepthAttachment::new_read_only(draw.targets.depth)),
      (
        WaterReflectionParameters {
          water_current: current,
          ..parameters
        },
        None,
      ),
      (&frame.args, frame.groups),
      |it| &it.reflection_resolve,
    );

    let half: (u32, u32) = (width.div_ceil(2), height.div_ceil(2));
    let across: GraphTexture = self.blur.add(
      graph,
      runtime,
      ("water blur across", written, half, WaterBlurUniform::ACROSS),
      enhanced,
    );
    let down: GraphTexture = self.blur.add(
      graph,
      runtime,
      ("water blur down", across, half, WaterBlurUniform::DOWN),
      enhanced,
    );

    (down, written)
  }

  /// Begins the surface's pass: over the scene, the distortion target and the motion, against the read-only depth it
  /// also samples.
  fn begin_surface_pass<'g, 'a>(graph: &'g mut FrameGraph<'a>, draw: &WaterDraw<'a>) -> RasterPassBuilder<'g, 'a> {
    graph
      .add_raster_pass("water")
      .color(GraphColorAttachment::new(draw.targets.scene, wgpu::LoadOp::Load))
      .color(GraphColorAttachment::new(draw.targets.distortion, wgpu::LoadOp::Load))
      .color(GraphColorAttachment::new(draw.targets.motion, wgpu::LoadOp::Load))
      .depth(GraphDepthAttachment::new_read_only(draw.targets.depth))
  }

  /// Adds a pass drawing every water batch from the cull's argument buffers, with its parameters bound, the request
  /// list its rays are left in where `listed` gives one, and the pipeline `pick` chooses.
  fn add_batch_pass<'a, P: PassParameters + Send + 'a>(
    &'a self,
    builder: RasterPassBuilder<'_, 'a>,
    (parameters, listed): (P, Option<TraceRequestParameters>),
    (args, groups): (&[GraphBuffer], WaterGroups<'a>),
    pick: impl Fn(&WaterPipelines) -> &wgpu::RenderPipeline + Send + 'a,
  ) {
    let args: Vec<GraphBuffer> = args.to_vec();
    let builder: RasterPassBuilder<'_, 'a> = match &listed {
      Some(listed) => builder.parameters(&parameters).parameters(listed),
      None => builder.parameters(&parameters),
    };

    args
      .iter()
      .fold(StaticDraws::declare_layouts(builder, &groups.2), |builder, args| {
        builder.buffer(*args, GraphBufferAccess::Indirect)
      })
      .record(move |context| {
        let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

        context.bind(&parameters);

        if let Some(listed) = &listed {
          context.bind(listed);
        }

        self.draw_batches(context, groups, &args, pick);
      });
  }

  /// Draws every water batch each argument buffer lists with the pipeline `pick` chooses, the pass's own group bound.
  fn draw_batches(
    &self,
    context: &mut RasterContext<'_>,
    (view, textures, layouts): WaterGroups<'_>,
    args: &[&wgpu::Buffer],
    pick: impl Fn(&WaterPipelines) -> &wgpu::RenderPipeline,
  ) {
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);
    context.get_pass().set_bind_group(1, textures, &[]);

    context.get_pass().set_pipeline(pick(&self.pipelines));

    for batch in StaticBatch::list_water() {
      context.bind(&layouts[batch.layout.get_index()]);

      for args in args {
        context.get_pass().draw_indirect(args, batch.get_args_offset());
      }
    }
  }

  /// The pipelines every batch draws with: the depth and the engine's surface from `static/water`, the enhanced surface
  /// from `static/water_enhanced`, each module with its own group 3; its reflection and resolve from
  /// `static/water_reflection`, with its own group 4 past the request list the reflection fills.
  fn create_pipelines(
    source: PipelineSource<'_>,
    [view_layout, texture_layout, scene_layout]: &[wgpu::BindGroupLayout; 3],
    request_layout: &wgpu::BindGroupLayout,
    parameters: &WaterParameterLayouts,
  ) -> XrfResult<WaterPipelines> {
    let engine_layouts: [Option<&wgpu::BindGroupLayout>; 4] = [
      Some(view_layout),
      Some(texture_layout),
      Some(scene_layout),
      Some(&parameters.engine),
    ];
    let enhanced_layouts: [Option<&wgpu::BindGroupLayout>; 4] = [
      Some(view_layout),
      Some(texture_layout),
      Some(scene_layout),
      Some(&parameters.enhanced),
    ];
    let depth_layouts: [Option<&wgpu::BindGroupLayout>; 4] = [
      Some(view_layout),
      Some(texture_layout),
      Some(scene_layout),
      Some(&parameters.depth),
    ];
    let reflection_layouts: [Option<&wgpu::BindGroupLayout>; 5] = [
      Some(view_layout),
      Some(texture_layout),
      Some(scene_layout),
      Some(request_layout),
      Some(&parameters.reflection),
    ];
    let resolve_layouts: [Option<&wgpu::BindGroupLayout>; 5] = [
      Some(view_layout),
      Some(texture_layout),
      Some(scene_layout),
      None,
      Some(&parameters.reflection),
    ];
    let blended = |format: wgpu::TextureFormat| {
      Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      })
    };
    // The surface's motion replaces what lies under it, which the resolve would carry the water's pixels by otherwise.
    let targets: [Option<wgpu::ColorTargetState>; 3] = [
      blended(ViewTargets::SCENE),
      blended(ViewTargets::DISTORTION),
      Some(ViewTargets::MOTION.into()),
    ];
    let reflection_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(WaterReflection::FORMAT.into())];
    let current_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(WaterReflection::CURRENT_FORMAT.into())];
    let describe = |label: &'static str,
                    module: &'static str,
                    bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
                    is_written: bool|
     -> RenderPipelineDescription {
      RenderPipelineDescription::new(label, module, "vs_water")
        .with_bind_group_layouts(bind_group_layouts)
        .with_primitive(wgpu::PrimitiveState {
          front_face: wgpu::FrontFace::Ccw,
          cull_mode: Some(wgpu::Face::Back),
          ..Default::default()
        })
        .with_depth_stencil(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(is_written),
          depth_compare: Some(wgpu::CompareFunction::Greater),
          stencil: Default::default(),
          bias: Default::default(),
        })
    };
    let surface = |(label, module, entry): (&'static str, &'static str, &'static str),
                   bind_group_layouts: &[Option<&wgpu::BindGroupLayout>],
                   targets: &[Option<wgpu::ColorTargetState>]|
     -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &describe(label, module, bind_group_layouts, false)
          .with_fragment(entry)
          .with_targets(targets.to_vec()),
      )
    };

    Ok(WaterPipelines {
      depth: source.get_render_pipeline(&describe("water depth", "static/water", &depth_layouts, true))?,
      reflection: surface(
        ("water reflection", "static/water_reflection", "fs_water_reflection"),
        &reflection_layouts,
        &current_targets,
      )?,
      reflection_resolve: surface(
        (
          "water reflection resolve",
          "static/water_reflection",
          "fs_water_resolve",
        ),
        &resolve_layouts,
        &reflection_targets,
      )?,
      engine: surface(("water", "static/water", "fs_water"), &engine_layouts, &targets)?,
      enhanced: surface(
        ("water enhanced", "static/water_enhanced", "fs_water_enhanced"),
        &enhanced_layouts,
        &targets,
      )?,
    })
  }
}
