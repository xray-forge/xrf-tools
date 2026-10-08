use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, StorageValue,
  UniformBinding,
};

use crate::context::gpu_requirements::can_trace;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::frame_texture::FrameTexture;
use crate::frame::reflection_frame::ReflectionFrame;
use crate::frame::reflection_history::ReflectionHistory;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::reflection_pass::ReflectionPass;
use crate::pass::reflection_scene_parameters::ReflectionSceneParameters;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::trace_request_handles::TraceRequestHandles;
use crate::pass::trace_request_parameters::TraceRequestParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The screen-space reflections, while they are traced: their settings and trace this frame, what their passes read,
/// and what they keep from frame to frame.
#[derive(Default)]
pub struct ReflectionFeature {
  settings: RenderReflectionSettings,
  trace: Option<ReflectionTrace>,
  uniform: ReflectionUniform,
  history: Option<ReflectionHistory>,
}

impl ReflectionFeature {
  /// Takes this frame's reflections for a frame `size` texels across: the history at the size they trace at and the
  /// pyramid over the frame's depth made or kept while they are traced, dropped otherwise, the trace's request list
  /// fitted, and what their passes read.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions, (width, height): (u32, u32)) {
    self.settings = options.features.reflections;
    self.trace = ReflectionTrace::new(options);

    let Some(trace) = self.trace else {
      self.history = None;

      return;
    };
    let ratio: u32 = trace.get_ratio();
    let traced: (u32, u32) = (width.div_ceil(ratio), height.div_ceil(ratio));

    if !self
      .history
      .as_ref()
      .is_some_and(|it| it.is_sized(traced, (width, height)))
    {
      self.history = Some(ReflectionHistory::new(device, traced, (width, height)));
    }

    let can_trace: bool = can_trace(device);

    if let Some(history) = &mut self.history {
      let is_traced: bool = self.settings.is_traced && can_trace;

      history.requests.fit(
        device,
        if is_traced {
          (traced.0 * traced.1).div_ceil(ReflectionHistory::REQUEST_SHARE)
        } else {
          0
        },
      );
    }

    let (frame, has_history, has_source): (u32, bool, bool) = self.history.as_ref().map_or((0, false, false), |it| {
      (it.frames.get_frame(), it.frames.is_valid(), it.is_source_valid)
    });

    self.uniform = ReflectionUniform::new(&self.settings, frame, (has_history, has_source), can_trace);
  }

  pub fn get_settings(&self) -> RenderReflectionSettings {
    self.settings
  }

  /// How they are traced this frame, none where they are not.
  pub fn get_trace(&self) -> Option<ReflectionTrace> {
    self.trace
  }

  /// Reduces the frame's `depth` as it stands now (before or after the grass drew) to its nearest, level by level, the
  /// first level the depth itself: what the rays are walked over, so a bush's billboards stand in for the volume they
  /// are and never stop a ray going for what lies behind them. Nothing where they are not traced.
  pub fn add_pyramid_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    depth: GraphTexture,
  ) {
    let (Some(_), Some(history)) = (self.trace, &self.history) else {
      return;
    };
    let pyramid: &'a DepthPyramid = &history.pyramid;
    let level_views: Vec<GraphTexture> = pyramid
      .level_views
      .iter()
      .map(|level| bindings.import_view(&mut *graph, "nearest depth level", level))
      .collect();
    let first: PyramidDepthParameters = PyramidDepthParameters {
      source_depth: depth,
      target_level: level_views[0],
    };
    let levels: Vec<PyramidLevelParameters> = level_views
      .windows(2)
      .map(|pair| PyramidLevelParameters {
        source_level: pair[0],
        target_level: pair[1],
      })
      .collect();

    levels
      .iter()
      .fold(
        graph.add_compute_pass(ReflectionTrace::PYRAMID_PASS).parameters(&first),
        |builder, level| builder.parameters(level),
      )
      .record(move |context| passes.pyramid.record(context, pyramid, (&first, &levels), true));
  }

  /// The last frame's scene and depth they keep, read by this frame's hits: the scene copied again after the particles
  /// draw, the depth the target the last frame drew into.
  pub fn import_source<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
  ) -> Option<(GraphTexture, GraphTexture)> {
    self.history.as_ref().map(|history| {
      (
        bindings.import_view(&mut *graph, "reflection source scene", &history.scene.view),
        bindings.import_view(&mut *graph, "reflection source depth", &history.depth),
      )
    })
  }

  /// Declares the screen-space reflections while they are traced: every glossy pixel's ray walked over the nearest
  /// depth's pyramid, then the last frame's reflection reprojected, an eighth-size average, the
  /// trace filtered across the surface and resolved over time into the view's history. Answers the history written,
  /// none where they are not traced.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, lit, wet_surface): (ViewTargetHandles, LightingHandles<'a>, GraphTexture),
    source: Option<(GraphTexture, GraphTexture)>,
    (sky, scene_trace): (&SkyParameters<'a>, Option<(TraceParameters<'a>, &'a wgpu::BindGroup)>),
  ) -> Option<(GraphTexture, GraphTexture)> {
    self.trace?;

    let history: &'a ReflectionHistory = self.history.as_ref()?;
    let (source_scene, source_depth): (GraphTexture, GraphTexture) = source?;
    let sky: SkyParameters<'a> = *sky;
    let reflection: UniformBinding<ReflectionUniform> = runtime.push_uniform(&self.uniform);
    let empty: GraphTexture = bindings.import_view(graph, "reflections empty", passes.reflections.get_empty());
    let blue_noise: GraphTexture =
      bindings.import_view(graph, "reflection blue noise", &passes.water.get_maps().blue_noise);
    let nearest_depth: GraphTexture = bindings.import_view(graph, "nearest depth pyramid", &history.pyramid.view);

    let (width, height) = history.size;
    let ratio: u32 = ReflectionPass::AVERAGE_RATIO;
    let mut create = |label: &'static str, (width, height): (u32, u32), format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    let traced: GraphTexture = create("reflections traced", (width, height), ReflectionPass::TRACED);
    let ends: GraphTexture = create("reflections ends", (width, height), ReflectionPass::ENDS);
    let reprojected: GraphTexture = create("reflections reprojected", (width, height), ReflectionPass::REPROJECTED);
    let variance: GraphTexture = create("reflections variance", (width, height), ReflectionPass::VARIANCE);
    let average: GraphTexture = create(
      "reflections average",
      (width.div_ceil(ratio), height.div_ceil(ratio)),
      ReflectionPass::AVERAGE,
    );
    let prefiltered: GraphTexture = create("reflections prefiltered", (width, height), ReflectionPass::PREFILTERED);
    let guide: GraphTexture = create("reflections guide", (width, height), ReflectionPass::GUIDED);
    let [written_frame, read_frame]: [&'a ReflectionFrame; 2] = history.frames.get_sides();
    let mut import_pair =
      |label: &'static str, texture: fn(&'a ReflectionFrame) -> &'a FrameTexture| -> [GraphTexture; 2] {
        [read_frame, written_frame].map(|frame| bindings.import_view(&mut *graph, label, &texture(frame).view))
      };
    let [previous, written] = import_pair("reflection history", |frame| &frame.radiance);
    let [previous_surface, written_surface] = import_pair("reflection history surface", |frame| &frame.surface);
    let [previous_held, written_held] = import_pair("reflection history held", |frame| &frame.held);
    let [previous_full, written_full] = import_pair("reflection history full", |frame| &frame.full);
    let parameters: ReflectionParameters = ReflectionParameters {
      albedo_target: targets.albedo,
      normal_target: targets.normal,
      material_target: targets.material,
      depth_target: targets.depth,
      light_target: targets.light,
      occlusion_target: targets.occlusion[0],
      wet_surface,
      material_lut: lit.material_lut,
      lut_sampler: lit.lut_sampler,
      lighting: lit.lighting,
      reflection,
      exposure: StorageValue::new(lit.exposure),
      motion_target: targets.motion,
      source_scene,
      source_depth,
      nearest_depth,
      blue_noise,
      traced: empty,
      reprojected: empty,
      variance: empty,
      average: empty,
      prefiltered: empty,
      history_radiance: previous,
      history_surface: previous_surface,
      history_held: previous_held,
      resolved: empty,
      history_full: empty,
      guide: empty,
      history_sampler: passes.reflections.get_sampler(),
    };
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));

    let listed: TraceRequestHandles =
      passes
        .scene_trace
        .add_reset(graph, bindings, ("reflection trace requests clear", &history.requests));
    let requests: TraceRequestParameters = listed.to_listed();

    graph
      .add_raster_pass(ReflectionTrace::TRACE_PASS)
      .parameters(&parameters)
      .parameters(&sky)
      .parameters(&requests)
      .color(clear(traced))
      .color(clear(ends))
      .record(move |context| {
        passes
          .reflections
          .record_trace(context, view, (&parameters, &sky), &requests)
      });

    // The rays the trace left, traced against the level's geometry.
    if let Some(scene_trace) = scene_trace {
      let (trace_requests, trace_request_head) = listed.to_read();

      passes.scene_trace.add_trace(
        graph,
        (("reflection scene dispatch", ReflectionTrace::SCENE_PASS), listed),
        view,
        (
          ReflectionSceneParameters {
            lighting: lit.lighting,
            material_lut: lit.material_lut,
            lut_sampler: lit.lut_sampler,
            exposure: StorageValue::new(lit.exposure),
            trace_requests,
            trace_request_head,
            traced_out: traced,
            ends_out: ends,
          },
          sky,
        ),
        scene_trace,
      );
    }

    // Each traced pixel's surface for the stages after the trace, worked out once.
    graph
      .add_raster_pass(ReflectionTrace::GUIDE_PASS)
      .parameters(&parameters)
      .color(clear(guide))
      .record(move |context| {
        passes
          .reflections
          .record_stage(context, view, &parameters, ReflectionPass::GUIDE)
      });

    let parameters: ReflectionParameters<'a> = ReflectionParameters { guide, ..parameters };
    // Each stage, what it reads beside the G-buffer and the history, and what it writes.
    let stages: [(&'static str, &'static str, ReflectionParameters<'a>, Vec<GraphTexture>); 5] = [
      (
        ReflectionTrace::REPROJECT_PASS,
        ReflectionPass::REPROJECT,
        ReflectionParameters { traced, ..parameters },
        vec![reprojected, variance],
      ),
      (
        ReflectionTrace::AVERAGE_PASS,
        ReflectionPass::AVERAGE_STAGE,
        ReflectionParameters {
          traced,
          reprojected,
          ..parameters
        },
        vec![average],
      ),
      (
        ReflectionTrace::PREFILTER_PASS,
        ReflectionPass::PREFILTER,
        ReflectionParameters {
          traced,
          variance,
          average,
          ..parameters
        },
        vec![prefiltered],
      ),
      (
        ReflectionTrace::RESOLVE_PASS,
        ReflectionPass::RESOLVE,
        ReflectionParameters {
          traced,
          reprojected,
          variance,
          average,
          prefiltered,
          ..parameters
        },
        vec![written, written_surface, written_held],
      ),
      (
        ReflectionTrace::UPSCALE_PASS,
        ReflectionPass::UPSCALE,
        ReflectionParameters {
          resolved: written,
          history_full: previous_full,
          ..parameters
        },
        vec![written_full],
      ),
    ];

    for (name, stage, stage_parameters, outputs) in stages {
      outputs
        .into_iter()
        .fold(
          graph.add_raster_pass(name).parameters(&stage_parameters),
          |builder, output| builder.color(clear(output)),
        )
        .record(move |context| passes.reflections.record_stage(context, view, &stage_parameters, stage));
    }

    Some((written_full, ends))
  }

  /// What this frame wrote becomes what the next reads, while they are traced: the view's depth target this frame drew
  /// into (`depth`) kept, and the one kept handed back for the view to draw into next.
  pub fn end_frame(&mut self, depth: &mut wgpu::TextureView) {
    if let (Some(_), Some(history)) = (self.trace, &mut self.history) {
      history.swap(depth);
    }
  }
}

impl ViewFeature for ReflectionFeature {
  /// Their history, the pyramid and the kept scene and depth.
  fn get_memory(&self) -> u64 {
    self.history.as_ref().map_or(0, ReflectionHistory::get_memory)
  }

  fn release(&mut self) {
    self.history = None;
  }
}
