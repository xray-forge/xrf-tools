use glam::{Mat4, UVec4, Vec2};
use xrf_math::EPS_S;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;

/// The static scene as the view culls and draws it: the cull's parameters this frame, the depth pyramid the next
/// frame's occlusion is tested against and the camera it was reduced through, the counts the cull read back, and
/// whether the wall marks are drawn.
pub struct StaticsFeature {
  cull: StaticCullParams,
  occlusion: StaticOcclusionUniform,
  /// The farthest depth's pyramid over the frame's targets.
  pyramid: Option<DepthPyramid>,
  /// The view and projection the pyramid holds a frame's depth through, once one was reduced.
  history: Option<(Mat4, Mat4)>,
  /// This frame's view and projection.
  matrices: (Mat4, Mat4),
  stats: StatsReadback,
  is_wallmarked: bool,
  /// Whether the view inspects its surfaces, which draws them with the inspections' code.
  is_inspected: bool,
}

impl StaticsFeature {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      cull: StaticCullParams::default(),
      occlusion: bytemuck::Zeroable::zeroed(),
      pyramid: None,
      history: None,
      matrices: (Mat4::IDENTITY, Mat4::IDENTITY),
      stats: StatsReadback::new(device),
      is_wallmarked: false,
      is_inspected: true,
    }
  }

  /// Makes the depth pyramid over targets `width` by `height`, after they were made again: a pyramid of another size
  /// holds no depth this frame can be tested against.
  pub fn fit(&mut self, device: &wgpu::Device, (width, height): (u32, u32)) {
    self.pyramid = Some(DepthPyramid::new(device, width, height));
    self.history = None;
  }

  /// Takes this frame's cull of `statics` through `view`, a lens `field_of_view` degrees tall over a frame `width` by
  /// `height`, and what the occlusion tests against.
  pub fn prepare(
    &mut self,
    statics: &StaticScene,
    options: &RenderViewOptions,
    (view, field_of_view): (&CameraView, f32),
    (width, height): (u32, u32),
  ) {
    if !options.features.is_occlusion_culled {
      self.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.features.lod.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.cull = StaticCullParams {
      cluster_count: statics.geometry.get_cluster_count(),
      row_count: statics.geometry.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: statics.geometry.get_impostor_count(),
      glod_start: threshold(options.features.lod.ssa_glod_start),
      glod_end: threshold(options.features.lod.ssa_glod_end),
      discard_below: options.features.lod.ssa_discard.powi(2) / screen,
      candidate_capacity: statics.cull.get_list_capacity(),
      is_occluding: options.features.is_occlusion_culled as u32,
      lod_a: threshold(options.features.lod.ssa_a),
      lod_b: threshold(options.features.lod.ssa_b),
      is_impostors: options.features.lod.is_impostors as u32,
      pad: UVec4::ZERO,
      lod_origin: view.position.extend(1.0),
    };
    self.matrices = (view.view, view.projection);
    self.is_wallmarked = options.show.is_wallmarked;
    self.is_inspected = options.is_inspecting_surfaces();

    if let Some(pyramid) = &self.pyramid {
      let (history_view, history_projection): (Mat4, Mat4) = self.history.unwrap_or(self.matrices);

      self.occlusion = StaticOcclusionUniform {
        view: history_view,
        projection: history_projection,
        size: Vec2::new(pyramid.width as f32, pyramid.height as f32),
        levels: pyramid.levels,
        has_history: self.history.is_some() as u32,
      };
    }
  }

  /// The cull's parameters this frame, which every cull of the view's own views reads.
  pub fn get_cull(&self) -> &StaticCullParams {
    &self.cull
  }

  /// What this frame's depth pyramid holds, for a cull after the G-buffer to test against: this frame's view, the
  /// pyramid's shape, and whether it was reduced this frame at all, which only an occlusion cull does.
  pub fn get_frame_occlusion(&self) -> StaticOcclusionUniform {
    StaticOcclusionUniform {
      view: self.matrices.0,
      projection: self.matrices.1,
      ..self.occlusion
    }
    .with_history(self.cull.is_occluding != 0)
  }

  /// Whether a frame can be drawn: the pyramid is made with the targets.
  pub fn is_ready(&self) -> bool {
    self.pyramid.is_some()
  }

  /// Keeps this frame's camera as what the pyramid it reduces holds, while it culls occlusion.
  pub fn begin_frame(&mut self) {
    if self.cull.is_occluding != 0 {
      self.history = Some(self.matrices);
    }
  }

  /// The static scene's handles for this frame: its buffers, the pyramid, the cull's parameters, what the occlusion
  /// tests against and the trees' sway pushed; none before the pyramid is made.
  pub fn import<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    statics: &'a StaticScene,
    wind: &WindUniform,
  ) -> Option<StaticSceneHandles> {
    let pyramid: &'a DepthPyramid = self.pyramid.as_ref()?;

    Some(StaticSceneHandles::import(
      (graph, bindings, runtime),
      statics,
      &pyramid.view,
      (&self.cull, &self.occlusion, wind),
    ))
  }

  /// The draw arguments a forward pass replays: the early phase's, and the late phase's where occlusion culls.
  pub fn list_draw_args(&self, scene: &StaticSceneHandles) -> Vec<GraphBuffer> {
    if self.cull.is_occluding != 0 {
      vec![scene.args, scene.late]
    } else {
      vec![scene.args]
    }
  }

  /// Declares the scene's culls and G-buffer draws: what last frame's depth does not hide, culled and drawn; then, while
  /// it culls occlusion, this frame's depth reduced and what the first draw does not hide of the rest culled and drawn.
  pub fn add_gbuffer_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &StaticSceneHandles),
    targets: ViewTargetHandles,
  ) {
    let Some(pyramid) = &self.pyramid else {
      return;
    };
    let params: &'a StaticCullParams = &self.cull;
    let cull: StaticCullParameters = scene.get_camera_cull();
    let draws: StaticDraws = scene.get_camera_draws();
    let gbuffer: [GraphTexture; 5] = targets.get_gbuffer();
    let depth: GraphTexture = targets.depth;
    let is_inspected: bool = self.is_inspected;
    let add_draw = |graph: &mut FrameGraph<'a>, name: &'static str, draw_args: GraphBuffer, is_first: bool| {
      let color_load: wgpu::LoadOp<wgpu::Color> = if is_first {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
      } else {
        wgpu::LoadOp::Load
      };
      let depth_load: wgpu::LoadOp<f32> = if is_first {
        wgpu::LoadOp::Clear(0.0)
      } else {
        wgpu::LoadOp::Load
      };
      let builder = gbuffer
        .iter()
        .fold(graph.add_raster_pass(name), |builder, texture| {
          builder.color(GraphColorAttachment::new(*texture, color_load))
        })
        .depth(GraphDepthAttachment::new(depth, depth_load))
        .buffer(draw_args, GraphBufferAccess::Indirect);

      draws.declare(builder, is_first).record(move |context| {
        let draw_args: &wgpu::Buffer = context.get_buffer(draw_args);

        passes
          .gbuffer
          .record(context, (view, &draws, textures), draw_args, (is_first, is_inspected));
      });
    };

    graph
      .add_compute_pass("cull")
      .parameters(&cull)
      .record(move |context| passes.cull.record_early(context, view, &cull, params));
    add_draw(graph, "g-buffer", scene.args, true);

    if self.cull.is_occluding == 0 {
      return;
    }

    let (late, late_dispatch): (GraphBuffer, GraphBuffer) = (scene.late, scene.late_dispatch);
    // Each level as its own resource, the depth reduced into the first and each into the next.
    let level_views: Vec<GraphTexture> = pyramid
      .level_views
      .iter()
      .map(|view| bindings.import_view(&mut *graph, "depth pyramid level", view))
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
        graph.add_compute_pass("depth pyramid").parameters(&first),
        |builder, level| builder.parameters(level),
      )
      .record(move |context| passes.pyramid.record(context, pyramid, (&first, &levels), false));
    graph
      .add_encoder_pass("late cull dispatch")
      .buffer(late, GraphBufferAccess::CopySource)
      .buffer(late_dispatch, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (late, dispatch) = (context.get_buffer(late), context.get_buffer(late_dispatch));

        passes.cull.record_late_dispatch(context.get_encoder(), late, dispatch);
      });
    graph
      .add_compute_pass("late cull")
      .buffer(late_dispatch, GraphBufferAccess::Indirect)
      .parameters(&cull)
      .record(move |context| {
        let dispatch: &wgpu::Buffer = context.get_buffer(late_dispatch);

        passes.cull.record_late(context, view, &cull, dispatch);
      });
    add_draw(graph, "late g-buffer", late, false);
  }

  /// Adds the cull's counts read back, for a report a frame or more later: an effect the graph cannot see.
  pub fn add_stats_pass<'a>(&'a self, graph: &mut FrameGraph<'a>, scene: &StaticSceneHandles) {
    let args: GraphBuffer = scene.args;

    graph
      .add_encoder_pass("stats")
      .buffer(args, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(args);

        self
          .stats
          .record(context.get_encoder(), args, StaticCullScratch::STATS_OFFSET);
      });
  }

  /// Adds the wall marks into the G-buffer's albedo before any light, where they are drawn.
  pub fn add_wallmark_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &StaticSceneHandles),
    targets: ViewTargetHandles,
  ) {
    if !self.is_wallmarked {
      return;
    }

    let args: Vec<GraphBuffer> = self.list_draw_args(scene);
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_camera_draws().layouts;

    StaticDraws::declare_layouts(
      args
        .iter()
        .fold(graph.add_raster_pass("wall marks"), |builder, args| {
          builder.buffer(*args, GraphBufferAccess::Indirect)
        })
        .color(GraphColorAttachment::new(targets.albedo, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth)),
      &layouts,
    )
    .record(move |context| {
      let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

      passes
        .composited
        .record_wallmarks(context, (view, &layouts, textures), &args);
    });
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.stats.request();
  }

  /// Drops the counts a frame that was never submitted copied out.
  pub fn abandon_stats(&self) {
    self.stats.abandon();
  }

  /// What the latest counted frame's cull kept and hid: clusters and triangles kept, then occluded.
  pub fn take_stats(&mut self) -> [u32; 4] {
    self.stats.take()
  }
}

impl ViewFeature for StaticsFeature {
  /// The depth pyramid.
  fn get_memory(&self) -> u64 {
    self.pyramid.as_ref().map_or(0, DepthPyramid::get_memory)
  }

  fn release(&mut self) {
    self.pyramid = None;
    self.history = None;
  }
}
