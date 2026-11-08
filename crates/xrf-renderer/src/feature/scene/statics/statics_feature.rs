use glam::{Mat4, UVec4, Vec2, Vec3, Vec4};
use xrf_math::EPS_S;
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDeclaration, GraphDepthAttachment,
  GraphTexture,
};

use crate::camera::camera_view::CameraView;
use crate::contract::view::output::render_debug_view::RenderDebugView;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::scene::pyramid::pyramid_reduction::PyramidReduction;
use crate::feature::scene::statics::cull::static_cull_parameters::StaticCullParameters;
use crate::feature::scene::statics::cull::static_cull_uniform::StaticCullUniform;
use crate::feature::scene::statics::cull::static_occlusion_uniform::StaticOcclusionUniform;
use crate::feature::scene::statics::cull::static_view_cull::StaticViewCull;
use crate::feature::scene::statics::draw::static_draw_bindings::StaticDrawBindings;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::scene::statics::draw::static_draws::StaticDraws;
use crate::feature::scene::statics::draw::static_layout_bindings::StaticLayoutBindings;
use crate::feature::scene::statics::gbuffer::static_gbuffer_draw::{StaticGbufferDraw, StaticGbufferPart};
use crate::feature::scene::wind::wind_uniform::WindUniform;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::readback::stats_readback::StatsReadback;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::targets::depth_pyramid::DepthPyramid;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::static_scene::draw::static_batch::StaticBatch;
use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;

/// What the camera's lens is scaled by for the frustum a sun cascade's trees are kept for: a third or so wider each way
/// at the usual lenses, so a camera turning between the trees' redraws still finds their shadows drawn.
const RECEIVER_WIDEN: f32 = 0.6;
/// The first G-buffer draw's passes, a part each (`StaticGbufferPart`): the depth drawn ahead, which clears the G-buffer,
/// the shading and the impostors; and the late draw's, which no impostor waits for.
const GBUFFER_PASSES: [&str; 3] = ["g-buffer depth", "g-buffer", "impostors"];
const LATE_GBUFFER_PASSES: [&str; 2] = ["late g-buffer depth", "late g-buffer"];

/// The static scene as the view culls and draws it: the cull's parameters this frame and the lists and arguments it
/// fills, the depth pyramid the next frame's occlusion is tested against and the camera it was reduced through, the
/// counts the cull read back, whether the wall marks are drawn, and whether its surfaces are counted for the overdraw's
/// debug view.
pub(crate) struct StaticsFeature {
  cull: StaticCullUniform,
  /// The camera cull's lists and arguments, the view's own, made with its first cull and as large as the scene's lists.
  camera: Option<StaticViewCull>,
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
  /// Whether the opaque surfaces' depth is drawn ahead of their shading this frame.
  is_prepassed: bool,
  /// Whether the view shows its overdraw, which draws its surfaces again to count them.
  is_overdrawn: bool,
}

impl StaticsFeature {
  pub(crate) fn new(device: &wgpu::Device) -> Self {
    Self {
      cull: StaticCullUniform::default(),
      camera: None,
      occlusion: bytemuck::Zeroable::zeroed(),
      pyramid: None,
      history: None,
      matrices: (Mat4::IDENTITY, Mat4::IDENTITY),
      stats: StatsReadback::new(device),
      is_wallmarked: false,
      is_inspected: true,
      is_prepassed: false,
      is_overdrawn: false,
    }
  }

  /// Makes the depth pyramid over targets `width` by `height`, after they were made again: a pyramid of another size
  /// holds no depth this frame can be tested against.
  pub(crate) fn fit(&mut self, device: &wgpu::Device, (width, height): (u32, u32)) {
    self.pyramid = Some(DepthPyramid::new(device, width, height));
    self.history = None;
  }

  /// Forgets the camera a frame that was never submitted kept as the pyramid's: the pyramid never reduced its depth.
  pub(crate) fn abandon_frame(&mut self) {
    self.history = None;
  }

  /// Takes this frame's cull of `statics` through `view` over a frame `width` by `height`, the lists it fills made as
  /// large as the scene's, and what the occlusion tests against.
  pub(crate) fn prepare(
    &mut self,
    device: &wgpu::Device,
    statics: &StaticScene,
    options: &RenderViewOptions,
    view: &CameraView,
    (width, height): (u32, u32),
  ) {
    if !options.features.is_occlusion_culled {
      self.history = None;
    }

    StaticViewCull::fit(&mut self.camera, device, statics.cull.get_list_bytes());

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees; parallel rays are
    // drawn at their finest whatever it is.
    let field_of_view: f32 = view.get_field_of_view().unwrap_or(90.0);
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.features.lod.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.cull = StaticCullUniform {
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
      lod_origin: view.position.extend(if view.is_orthographic() { 0.0 } else { 1.0 }),
      receivers: Self::get_receivers(view),
    };
    self.matrices = (view.view, view.projection);
    self.is_wallmarked = options.show.is_wallmarked;
    self.is_inspected = options.is_inspecting_surfaces();
    // An inspection cuts surfaces the depth pass would draw whole, a wireframe the faces between its edges.
    self.is_prepassed = options.features.is_depth_prepassed && !self.is_inspected;
    self.is_overdrawn = options.mode.debug_view == RenderDebugView::Overdraw;

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

  /// The planes a sun cascade's trees are kept within (`StaticCullUniform::receivers`): the camera's frustum through a
  /// lens `RECEIVER_WIDEN` wider, its sides and the plane through the eye, the last none.
  fn get_receivers(view: &CameraView) -> [Vec4; 6] {
    let mut projection: Mat4 = view.projection;

    projection.x_axis.x *= RECEIVER_WIDEN;
    projection.y_axis.y *= RECEIVER_WIDEN;

    let matrix: Mat4 = projection * view.view;
    let rows: [Vec4; 4] = [matrix.row(0), matrix.row(1), matrix.row(2), matrix.row(3)];
    let forward: Vec3 = (-view.view.inverse().z_axis.truncate()).normalize_or_zero();
    let side = |plane: Vec4| plane / plane.truncate().length().max(f32::EPSILON);

    [
      side(rows[3] + rows[0]),
      side(rows[3] - rows[0]),
      side(rows[3] + rows[1]),
      side(rows[3] - rows[1]),
      forward.extend(-forward.dot(view.position)),
      Vec4::ZERO,
    ]
  }

  /// The cull's parameters this frame, which every cull of the view's own views reads.
  pub(crate) fn get_cull(&self) -> &StaticCullUniform {
    &self.cull
  }

  /// What this frame's depth pyramid holds, for a cull after the G-buffer to test against: this frame's view, the
  /// pyramid's shape, and whether it was reduced this frame at all, which only an occlusion cull does.
  pub(crate) fn get_frame_occlusion(&self) -> StaticOcclusionUniform {
    StaticOcclusionUniform {
      view: self.matrices.0,
      projection: self.matrices.1,
      ..self.occlusion
    }
    .with_history(self.cull.is_occluding != 0)
  }

  /// Whether a frame can be drawn: the pyramid is made with the targets.
  pub(crate) fn is_ready(&self) -> bool {
    self.pyramid.is_some()
  }

  /// Keeps this frame's camera as what the pyramid it reduces holds, while it culls occlusion.
  pub(crate) fn begin_frame(&mut self) {
    if self.cull.is_occluding != 0 {
      self.history = Some(self.matrices);
    }
  }

  /// The static scene's handles for this frame: its buffers, the view's lists and arguments, the pyramid, the cull's
  /// parameters, what the occlusion tests against and the trees' sway pushed; none before the pyramid is made or the
  /// view first culled.
  pub(crate) fn import<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    statics: &'a StaticScene,
    wind: &WindUniform,
  ) -> Option<StaticSceneHandles> {
    let pyramid: &'a DepthPyramid = self.pyramid.as_ref()?;

    Some(StaticSceneHandles::import(
      GraphDeclaration::new(graph, bindings, runtime),
      statics,
      self.camera.as_ref()?,
      &pyramid.view,
      &self.cull,
      &self.occlusion,
      wind,
    ))
  }

  /// The draw arguments a forward pass replays: the early phase's, and the late phase's where occlusion culls.
  pub(crate) fn list_draw_args(&self, scene: &StaticSceneHandles) -> Vec<GraphBuffer> {
    if self.cull.is_occluding != 0 {
      vec![scene.args, scene.late]
    } else {
      vec![scene.args]
    }
  }

  /// Declares the overdraw's count where the view shows it: the frame's draws replayed into `light`, which the sun and
  /// the lamps left nought, each surface adding one over every pixel it covers.
  pub(crate) fn add_overdraw_pass<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    ViewRecord {
      passes,
      view,
      textures,
      statics,
      ..
    }: ViewRecord<'a>,
    light: GraphTexture,
  ) {
    let scene: &StaticSceneHandles = &statics;
    if !self.is_overdrawn {
      return;
    }

    let draws: StaticDraws = scene.get_camera_draws();
    let draw_args: Vec<GraphBuffer> = self.list_draw_args(scene);
    let builder = draw_args.iter().fold(
      graph
        .add_raster_pass("overdraw")
        .color(GraphColorAttachment::new(light, wgpu::LoadOp::Load)),
      |builder, args| builder.buffer(*args, GraphBufferAccess::Indirect),
    );

    draws.declare(builder).record(move |context| {
      let args: Vec<&wgpu::Buffer> = draw_args.iter().map(|args| context.get_buffer(*args)).collect();

      passes
        .scene
        .gbuffer
        .record_overdraw(context, StaticDrawBindings::new(view, &draws, textures), &args);
    });
  }

  /// Declares the scene's culls and G-buffer draws: what last frame's depth does not hide, culled and drawn; then,
  /// while it culls occlusion, this frame's depth reduced and what the first draw does not hide of the rest culled and
  /// drawn.
  pub(crate) fn add_gbuffer_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      view,
      textures,
      handles: targets,
      statics,
      ..
    }: ViewRecord<'a>,
  ) {
    let scene: &StaticSceneHandles = &statics;
    let Some(pyramid) = &self.pyramid else {
      return;
    };
    let params: &'a StaticCullUniform = &self.cull;
    let cull: StaticCullParameters = scene.get_camera_cull();
    let draws: StaticDraws = scene.get_camera_draws();
    let gbuffer: [GraphTexture; 5] = targets.get_gbuffer();
    let depth: GraphTexture = targets.depth;
    let is_inspected: bool = self.is_inspected;
    let is_prepassed: bool = self.is_prepassed;
    // A pass a part of a draw, so the frame times them apart, the frame's first clearing the G-buffer.
    let add_part = |graph: &mut FrameGraph<'a>,
                    name: &'static str,
                    draw_args: GraphBuffer,
                    (part, is_clearing): (StaticGbufferPart, bool)| {
      let color_load: wgpu::LoadOp<wgpu::Color> = if is_clearing {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
      } else {
        wgpu::LoadOp::Load
      };
      let depth_load: wgpu::LoadOp<f32> = if is_clearing {
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
      let builder = if part == StaticGbufferPart::Impostors {
        draws.declare_impostored(builder)
      } else {
        draws.declare(builder)
      };

      builder.record(move |context| {
        let draw_args: &wgpu::Buffer = context.get_buffer(draw_args);

        passes.scene.gbuffer.record(
          context,
          StaticDrawBindings::new(view, &draws, textures),
          draw_args,
          StaticGbufferDraw {
            is_inspected,
            is_prepassed,
            part,
          },
        );
      });
    };
    // A draw's parts in order, as many as it names passes: the first draw draws the impostors too, as no occlusion sets
    // one aside, and its depth clears the G-buffer.
    let add_draw = |graph: &mut FrameGraph<'a>, names: &[&'static str], draw_args: GraphBuffer, is_first: bool| {
      let parts = [
        StaticGbufferPart::Depth,
        StaticGbufferPart::Shading,
        StaticGbufferPart::Impostors,
      ];

      for (index, (name, part)) in names.iter().zip(parts).enumerate() {
        add_part(&mut *graph, name, draw_args, (part, is_first && index == 0));
      }
    };

    // The view's arguments start each frame's cull empty, from the scene's templates.
    let reset: [GraphBuffer; 4] = [scene.args_template, scene.args, scene.late_template, scene.late];

    graph
      .add_encoder_pass("cull reset")
      .buffer(reset[0], GraphBufferAccess::CopySource)
      .buffer(reset[1], GraphBufferAccess::CopyDestination)
      .buffer(reset[2], GraphBufferAccess::CopySource)
      .buffer(reset[3], GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let [args_template, args, late_template, late] = reset.map(|buffer| context.get_buffer(buffer));
        let encoder: &mut wgpu::CommandEncoder = context.get_encoder();

        encoder.copy_buffer_to_buffer(args_template, 0, args, 0, StaticCullScratch::ARGS_SIZE);
        encoder.copy_buffer_to_buffer(late_template, 0, late, 0, StaticCullScratch::LATE_SIZE);
      });
    graph
      .add_compute_pass("cull")
      .parameters(&cull)
      .record(move |context| passes.scene.cull.record_early(context, view, &cull, params));
    add_draw(graph, &GBUFFER_PASSES, scene.args, true);

    if self.cull.is_occluding == 0 {
      return;
    }

    let (late, late_dispatch): (GraphBuffer, GraphBuffer) = (scene.late, scene.late_dispatch);
    passes.scene.pyramid.add_passes(
      &mut *graph,
      &mut *bindings,
      "depth pyramid",
      (pyramid, depth),
      PyramidReduction::Farthest,
    );
    graph
      .add_encoder_pass("late cull dispatch")
      .buffer(late, GraphBufferAccess::CopySource)
      .buffer(late_dispatch, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (late, dispatch) = (context.get_buffer(late), context.get_buffer(late_dispatch));

        passes
          .scene
          .cull
          .record_late_dispatch(context.get_encoder(), late, dispatch);
      });
    graph
      .add_compute_pass("late cull")
      .buffer(late_dispatch, GraphBufferAccess::Indirect)
      .parameters(&cull)
      .record(move |context| {
        let dispatch: &wgpu::Buffer = context.get_buffer(late_dispatch);

        passes.scene.cull.record_late(context, view, &cull, dispatch);
      });
    add_draw(graph, &LATE_GBUFFER_PASSES, late, false);
  }

  /// Adds the cull's counts read back, for a report a frame or more later: an effect the graph cannot see.
  pub(crate) fn add_stats_pass<'a>(&'a self, graph: &mut FrameGraph<'a>, scene: &StaticSceneHandles) {
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
  pub(crate) fn add_wallmark_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    ViewRecord {
      passes,
      view,
      textures,
      handles: targets,
      statics,
      ..
    }: ViewRecord<'a>,
  ) {
    let scene: &StaticSceneHandles = &statics;
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
        .effects
        .composited
        .record_wallmarks(context, StaticLayoutBindings::new(view, &layouts, textures), &args);
    });
  }

  /// The camera cull's draw arguments, its counts after them (`StaticCullScratch::STATS_OFFSET`), once it first culled.
  #[cfg(test)]
  pub(crate) fn get_camera_args(&self) -> Option<&wgpu::Buffer> {
    self.camera.as_ref().map(|camera| &camera.args)
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub(crate) fn request_stats(&self) {
    self.stats.request();
  }

  /// Drops the counts a frame that was never submitted copied out.
  pub(crate) fn abandon_stats(&self) {
    self.stats.abandon();
  }

  /// What the latest counted frame's cull kept and hid: clusters and triangles kept, then occluded.
  pub(crate) fn take_stats(&mut self) -> [u32; 4] {
    self.stats.take()
  }
}

impl ViewFeature for StaticsFeature {
  fn release(&mut self) {
    self.camera = None;
    self.pyramid = None;
    self.history = None;
  }
}
