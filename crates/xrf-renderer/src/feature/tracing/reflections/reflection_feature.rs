use xrf_renderer_core::{
  FrameGraph, GraphBufferAccess, GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor,
  StorageValue, UniformBinding,
};

use crate::context::gpu_requirements::can_trace;
use crate::contract::feature::tracing::render_reflection_settings::RenderReflectionSettings;
use crate::contract::feature::tracing::render_trace_size::RenderTraceSize;
use crate::contract::report::applied::render_applied_reflections::RenderAppliedReflections;
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::feature::image::resolve::resolve_decisions::ResolveDecisions;
use crate::feature::scene::pyramid::pyramid_reduction::PyramidReduction;
use crate::feature::tracing::reflections::draw::reflection_parameters::ReflectionParameters;
use crate::feature::tracing::reflections::draw::reflection_pass::ReflectionPass;
use crate::feature::tracing::reflections::draw::reflection_stage::ReflectionStage;
use crate::feature::tracing::reflections::history::reflection_frame::ReflectionFrame;
use crate::feature::tracing::reflections::history::reflection_frame_handles::ReflectionFrameHandles;
use crate::feature::tracing::reflections::history::reflection_history::ReflectionHistory;
use crate::feature::tracing::reflections::reflection_source::ReflectionSource;
use crate::feature::tracing::reflections::reflection_uniform::ReflectionUniform;
use crate::feature::tracing::reflections::reflections_recorded::ReflectionsRecorded;
use crate::feature::tracing::reflections::trace::budgeted_trace_outputs::BudgetedTraceOutputs;
use crate::feature::tracing::reflections::trace::reflection_trace::ReflectionTrace;
use crate::feature::tracing::reflections::trace::reflection_trace_frame::ReflectionTraceFrame;
use crate::feature::tracing::reflections::trace::reflection_trace_out_parameters::ReflectionTraceOutParameters;
use crate::feature::tracing::scene_trace::reflection_scene_parameters::ReflectionSceneParameters;
use crate::feature::tracing::scene_trace::requests::trace_request_handles::TraceRequestHandles;
use crate::feature::tracing::scene_trace::requests::trace_request_parameters::TraceRequestParameters;
use crate::feature::tracing::scene_trace::scene_trace_bindings::SceneTraceBindings;
use crate::feature::tracing::scene_trace::scene_trace_request::SceneTraceRequest;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::view::view_frame_end::ViewFrameEnd;
use crate::feature::view::view_restart::ViewRestart;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::feature::weather::wet::wet_feature::WetFeature;
use crate::feature::weather::wet::wet_targets::WetTargets;
use crate::frame::readback::stats_readback::StatsReadback;
use crate::frame::targets::frame_texture::FrameTexture;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_prepare::ViewPrepare;

/// The screen-space reflections, while they are traced: their settings and trace this frame, what their passes read,
/// and what they keep from frame to frame.
#[derive(Default)]
pub struct ReflectionFeature {
  settings: RenderReflectionSettings,
  trace: Option<ReflectionTrace>,
  uniform: ReflectionUniform,
  history: Option<ReflectionHistory>,
  /// The rays a raw trace asked for, read back a frame or more late, which its request list is sized to.
  demand: Option<StatsReadback>,
}

impl ReflectionFeature {
  /// Takes this frame's reflections for the frame as drawn, raw where the view's `resolve` decided DLSS reconstructs
  /// rays, the dry surfaces left untraced unless asked where the level is not wet (`wet`): the history at the size they
  /// trace at and the pyramid over the frame's depth made or kept while they are traced, dropped otherwise, the
  /// trace's request list fitted, and what their passes read.
  pub fn prepare(
    &mut self,
    &ViewPrepare {
      device,
      drawn: (width, height),
      options,
      ..
    }: &ViewPrepare<'_>,
    resolve: ResolveDecisions,
    wet: &WetFeature,
  ) {
    let (is_reconstructed, is_wet): (bool, bool) = (resolve.is_ray_reconstructed, wet.is_wet());
    self.settings = options.features.reflections;
    self.trace = ReflectionTrace::new(options).map(|trace| trace.with_raw(is_reconstructed));

    let Some(trace) = self.trace else {
      self.history = None;

      return;
    };
    let traced: (u32, u32) = ReflectionHistory::get_traced_size(trace, (width, height));

    if !self
      .history
      .as_ref()
      .is_some_and(|it| it.is_sized(trace, (width, height)))
    {
      self.history = Some(ReflectionHistory::new(device, trace, (width, height)));
    }

    let can_trace: bool = can_trace(device);

    let asked: u32 = self.demand.get_or_insert_with(|| StatsReadback::new(device)).take()[0];

    if let Some(history) = &mut self.history {
      let is_traced: bool = self.settings.is_traced && can_trace;
      let pixels: u32 = traced.0 * traced.1;
      let capacity: u32 = if !is_traced {
        0
      } else if trace.is_raw {
        ReflectionHistory::get_raw_capacity(pixels, history.requests.get_capacity(), asked)
      } else {
        pixels.div_ceil(ReflectionHistory::REQUEST_SHARE)
      };

      history.requests.fit(device, capacity);
    }

    let (frame, has_history, has_source): (u32, bool, bool) = self
      .history
      .as_ref()
      .map_or((0, false, false), |it| (it.frame, it.has_history(), it.is_source_valid));

    self.uniform = ReflectionUniform::new(
      &self.settings,
      trace,
      ReflectionTraceFrame {
        frame,
        has_history,
        has_source,
        can_trace,
        is_wet,
      },
    );
  }

  pub fn get_settings(&self) -> RenderReflectionSettings {
    self.settings
  }

  /// How they are traced this frame, none where they are not.
  pub fn get_trace(&self) -> Option<ReflectionTrace> {
    self.trace
  }

  /// Writes into `applied` how they are traced this frame, none where they are not: their strength, the share of the
  /// frame traced, full where the raw signal is, and the steps a ray walks.
  pub fn describe(&self, applied: &mut RenderAppliedReport) {
    applied.reflections = self.trace.map(|trace| RenderAppliedReflections {
      intensity: trace.intensity,
      trace_size: if trace.is_raw {
        RenderTraceSize::Full
      } else {
        trace.trace_size
      },
      steps: self.settings.get_steps(),
    });
  }

  /// Reduces the frame's `depth` as it stands now (before or after the grass drew) to its nearest, level by level, the
  /// first level the depth itself: what the rays are walked over, so a bush's billboards stand in for the volume they
  /// are and never stop a ray going for what lies behind them. Nothing where they are not traced.
  pub fn add_pyramid_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    depth: GraphTexture,
  ) {
    let (Some(_), Some(history)) = (self.trace, &self.history) else {
      return;
    };
    passes.scene.pyramid.add_passes(
      graph,
      bindings,
      ReflectionTrace::PYRAMID_PASS,
      (&history.pyramid, depth),
      PyramidReduction::Nearest,
    );
  }

  /// The last frame's scene and depth they keep, read by this frame's hits: the scene copied again after the particles
  /// draw, the depth the target the last frame drew into.
  pub fn import_source<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
  ) -> Option<ReflectionSource> {
    self.history.as_ref().map(|history| ReflectionSource {
      scene: bindings.import_view(&mut *graph, "reflection source scene", &history.scene.view),
      depth: bindings.import_view(&mut *graph, "reflection source depth", &history.depth),
    })
  }

  /// Declares the screen-space reflections while they are traced: every glossy pixel's ray walked over the nearest
  /// depth's pyramid, then, denoised, the last frame's reflection reprojected, an eighth-size average, the trace
  /// filtered across the surface and resolved over time into the view's history. Answers what they hand on, none where
  /// they are not traced.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    wet: WetTargets,
    source: Option<ReflectionSource>,
    sky: &SkyParameters<'a>,
    scene_trace: Option<TracedScene<'a>>,
  ) -> Option<ReflectionsRecorded> {
    self.trace?;

    let ViewRecord {
      passes,
      view,
      handles: targets,
      lit,
      ..
    } = record;
    let history: &'a ReflectionHistory = self.history.as_ref()?;
    let ReflectionSource {
      scene: source_scene,
      depth: source_depth,
    } = source?;
    let sky: SkyParameters<'a> = *sky;
    let reflection: UniformBinding<ReflectionUniform> = runtime.push_uniform(&self.uniform);
    let empty: GraphTexture = bindings.import_view(graph, "reflections empty", passes.tracing.reflections.get_empty());
    let blue_noise: GraphTexture = bindings.import_view(
      graph,
      "reflection blue noise",
      &passes.weather.water.get_maps().blue_noise,
    );
    let nearest_depth: GraphTexture = bindings.import_view(graph, "nearest depth pyramid", &history.pyramid.view);

    let (width, height) = history.size;
    let mut create = |label: &'static str, (width, height): (u32, u32), format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    let traced: GraphTexture = create("reflections traced", (width, height), ReflectionPass::TRACED);
    let ends: GraphTexture = create("reflections ends", (width, height), ReflectionPass::ENDS);
    let frames: Option<ReflectionFrameHandles> =
      Self::import_frames(GraphDeclaration::new(graph, bindings, runtime), history);
    let read: [GraphTexture; 3] = frames.as_ref().map_or([empty; 3], |frames| {
      [frames.radiance[0], frames.surface[0], frames.held[0]]
    });
    let parameters: ReflectionParameters = ReflectionParameters {
      albedo_target: targets.albedo,
      normal_target: targets.normal,
      material_target: targets.material,
      depth_target: targets.depth,
      light_target: targets.light,
      occlusion_target: targets.occlusion[0],
      wet_surface: wet.surface,
      wet_reach: wet.reach,
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
      history_radiance: read[0],
      history_surface: read[1],
      history_held: read[2],
      resolved: empty,
      history_full: empty,
      guide: empty,
      history_sampler: passes.tracing.reflections.get_sampler(),
    };
    self.add_trace_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      &parameters,
      sky,
      ReflectionTraceOutParameters {
        traced_out: traced,
        ends_out: ends,
      },
      scene_trace,
    );

    // Raw, DLSS ray reconstruction denoises the trace in the frame, so combine reads it as traced.
    let Some(frames) = frames else {
      return Some(ReflectionsRecorded {
        radiance: traced,
        ends,
        trace: traced,
      });
    };

    let written_full: GraphTexture =
      Self::add_denoise_passes(graph, passes, view, parameters, traced, frames, history.size);

    Some(ReflectionsRecorded {
      radiance: written_full,
      ends,
      trace: traced,
    })
  }

  /// Walks every glossy pixel's ray over the nearest depth's pyramid into `out`'s targets, then the rays the screen
  /// left against the level's geometry where the scene is traced (`scene_trace`); raw, how many rays the trace asked
  /// for read back for its list's room.
  fn add_trace_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord { passes, view, lit, .. }: ViewRecord<'a>,
    parameters: &ReflectionParameters<'a>,
    sky: SkyParameters<'a>,
    out: ReflectionTraceOutParameters,
    scene_trace: Option<TracedScene<'a>>,
  ) {
    let Some(history) = self.history.as_ref() else {
      return;
    };
    let ReflectionTraceOutParameters {
      traced_out: traced,
      ends_out: ends,
    } = out;
    let parameters: ReflectionParameters<'a> = *parameters;
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));

    let listed: TraceRequestHandles =
      passes
        .tracing
        .scene_trace
        .add_reset(graph, bindings, "reflection trace requests clear", &history.requests);
    let requests: TraceRequestParameters = listed.to_listed();

    if self.settings.is_ray_budgeted {
      let size: (u32, u32) = history.size;

      graph
        .add_compute_pass(ReflectionTrace::TRACE_PASS)
        .parameters(&parameters)
        .parameters(&sky)
        .parameters(&requests)
        .parameters(&out)
        .record(move |context| {
          passes.tracing.reflections.record_trace_budgeted(
            context,
            view,
            &parameters,
            &sky,
            BudgetedTraceOutputs {
              requests: &requests,
              out: &out,
            },
            size,
          )
        });
    } else {
      graph
        .add_raster_pass(ReflectionTrace::TRACE_PASS)
        .parameters(&parameters)
        .parameters(&sky)
        .parameters(&requests)
        .color(clear(traced))
        .color(clear(ends))
        .record(move |context| {
          passes
            .tracing
            .reflections
            .record_trace(context, view, &parameters, &sky, &requests)
        });
    }

    // The rays the trace left, traced against the level's geometry.
    if let Some(TracedScene { trace, textures }) = scene_trace {
      let head = listed.head;

      passes.tracing.scene_trace.add_trace(
        graph,
        SceneTraceRequest {
          dispatch_label: "reflection scene dispatch",
          trace_label: ReflectionTrace::SCENE_PASS,
          handles: listed,
        },
        SceneTraceBindings {
          view,
          parameters: ReflectionSceneParameters {
            lighting: lit.lighting,
            material_lut: lit.material_lut,
            lut_sampler: lit.lut_sampler,
            exposure: StorageValue::new(lit.exposure),
            traced_out: traced,
            ends_out: ends,
            screen_scene: parameters.source_scene,
            screen_depth: parameters.source_depth,
          },
          sky,
          trace,
          textures,
        },
      );

      // Raw, how many rays the trace asked for, read back for its list's room: an effect the graph cannot see.
      if let (Some(demand), true) = (&self.demand, self.trace.is_some_and(|it| it.is_raw)) {
        graph
          .add_encoder_pass("reflection request demand")
          .buffer(head, GraphBufferAccess::CopySource)
          .keep()
          .record(move |context| {
            let head: &wgpu::Buffer = context.get_buffer(head);

            demand.record(context.get_encoder(), head, 0);
          });
      }
    }
  }

  /// The trace denoised: each traced pixel's surface worked out once, then the last frame's reflection reprojected, an
  /// eighth-size average, the trace filtered across the surface, resolved over time into the history's `frames`, and
  /// brought up to the frame's size. Answers the history written at the frame's size.
  fn add_denoise_passes<'a>(
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    parameters: ReflectionParameters<'a>,
    traced: GraphTexture,
    frames: ReflectionFrameHandles,
    (width, height): (u32, u32),
  ) -> GraphTexture {
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));
    let ratio: u32 = ReflectionPass::AVERAGE_RATIO;
    let mut create = |label: &'static str, (width, height): (u32, u32), format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    let reprojected: GraphTexture = create("reflections reprojected", (width, height), ReflectionPass::REPROJECTED);
    let variance: GraphTexture = create("reflections variance", (width, height), ReflectionPass::VARIANCE);
    let average: GraphTexture = create(
      "reflections average",
      (width.div_ceil(ratio), height.div_ceil(ratio)),
      ReflectionPass::AVERAGE,
    );
    let prefiltered: GraphTexture = create("reflections prefiltered", (width, height), ReflectionPass::PREFILTERED);
    let guide: GraphTexture = create("reflections guide", (width, height), ReflectionPass::GUIDED);
    let [_, written] = frames.radiance;
    let [_, written_surface] = frames.surface;
    let [_, written_held] = frames.held;
    let [previous_full, written_full] = frames.full;

    // Each traced pixel's surface for the stages after the trace, worked out once.
    graph
      .add_raster_pass(ReflectionTrace::GUIDE_PASS)
      .parameters(&parameters)
      .color(clear(guide))
      .record(move |context| {
        passes
          .tracing
          .reflections
          .record_stage(context, view, &parameters, ReflectionStage::Guide)
      });

    let parameters: ReflectionParameters<'a> = ReflectionParameters { guide, ..parameters };
    // Each stage, what it reads beside the G-buffer and the history, and what it writes.
    let stages: [(
      &'static str,
      ReflectionStage,
      ReflectionParameters<'a>,
      Vec<GraphTexture>,
    ); 5] = [
      (
        ReflectionTrace::REPROJECT_PASS,
        ReflectionStage::Reproject,
        ReflectionParameters { traced, ..parameters },
        vec![reprojected, variance],
      ),
      (
        ReflectionTrace::AVERAGE_PASS,
        ReflectionStage::Average,
        ReflectionParameters {
          traced,
          reprojected,
          ..parameters
        },
        vec![average],
      ),
      (
        ReflectionTrace::PREFILTER_PASS,
        ReflectionStage::Prefilter,
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
        ReflectionStage::Resolve,
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
        ReflectionStage::Upscale,
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
        .record(move |context| {
          passes
            .tracing
            .reflections
            .record_stage(context, view, &stage_parameters, stage)
        });
    }

    written_full
  }

  /// The denoiser's history as the frame's graph holds it, the side read and the side written of each, while it is
  /// denoised.
  fn import_frames<'a>(
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    history: &'a ReflectionHistory,
  ) -> Option<ReflectionFrameHandles> {
    history.frames.as_ref().map(|frames| {
      let [written_frame, read_frame]: [&'a ReflectionFrame; 2] = frames.get_sides();
      let mut import_pair =
        |label: &'static str, texture: fn(&'a ReflectionFrame) -> &'a FrameTexture| -> [GraphTexture; 2] {
          [read_frame, written_frame].map(|frame| bindings.import_view(&mut *graph, label, &texture(frame).view))
        };

      ReflectionFrameHandles {
        radiance: import_pair("reflection history", |frame| &frame.radiance),
        surface: import_pair("reflection history surface", |frame| &frame.surface),
        held: import_pair("reflection history held", |frame| &frame.held),
        full: import_pair("reflection history full", |frame| &frame.full),
      }
    })
  }

  /// Asks for the rays asked recorded with the frame just submitted.
  pub fn request_demand(&self) {
    if let Some(demand) = &self.demand {
      demand.request();
    }
  }

  /// Drops the rays asked recorded with a frame that was never submitted.
  pub fn abandon_demand(&self) {
    if let Some(demand) = &self.demand {
      demand.abandon();
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

  /// What this frame wrote becomes what the next reads, while they are traced: the view's depth target this frame drew
  /// into kept, and the one kept handed back for the view to draw into next.
  fn end_frame(&mut self, end: &mut ViewFrameEnd<'_>) {
    if let (Some(_), Some(history), Some(depth)) = (self.trace, &mut self.history, end.depth.as_deref_mut()) {
      history.swap(depth);
    }
  }

  /// Their history starts again as `restart` asks (`ReflectionHistory::restart`).
  fn restart(&mut self, restart: ViewRestart) {
    if let Some(history) = &mut self.history {
      history.restart(restart);
    }
  }
}
