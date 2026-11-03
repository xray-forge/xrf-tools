use std::collections::HashMap;
use std::time::{Duration, Instant};

use glam::{Mat4, Vec3};
use xrf_error::XrfError;
use xrf_renderer_core::{ExecutedGroup, GraphOwner, GraphPassTime};

use crate::capture::burst::burst_job::BurstJob;
use crate::capture::capture_job::CaptureJob;
use crate::contract::feature::lighting::model::render_light_scales::RenderLightScales;
use crate::contract::overlay::render_overlay::RenderOverlay;
use crate::contract::overlay::render_selection::RenderSelection;
use crate::contract::report::applied::render_applied_environment::RenderAppliedEnvironment;
use crate::contract::report::applied::render_applied_fog::RenderAppliedFog;
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::contract::report::frame::render_frame_report::RenderFrameReport;
use crate::contract::report::frame::render_memory_report::RenderMemoryReport;
use crate::contract::report::frame::render_pass_cost::RenderPassCost;
use crate::contract::report::graph::render_graph_report::RenderGraphReport;
use crate::contract::report::load::render_load_report::RenderLoadReport;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_level_hit::RenderLevelHit;
use crate::contract::viewport::render_pick::RenderPick;
use crate::contract::viewport::render_rect::RenderRect;
use crate::contract::viewport::render_viewport_event::RenderViewportEvent;
use crate::contract::viewport::render_viewport_id::RenderViewportId;
use crate::contract::viewport::render_viewport_layout::RenderViewportLayout;
use crate::frame::readback::frame_capture::{CaptureReply, FrameCapture};
use crate::frame::readback::gpu_memory::GpuMemory;
use crate::frame::timing::frame_phases::FramePhases;
use crate::frame::timing::frame_statistics::FrameStatistics;
use crate::frame::timing::frame_summary::FrameSummary;
use crate::host::render_event_sink::RenderEventSink;
use crate::host::world::render_scene_id::RenderSceneId;
use crate::host::world::render_view_frame::RenderViewFrame;
use crate::lighting::weather::render_lighting::RenderLighting;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::view::scene_view::SceneView;
use crate::scene::level::view::view_reports::ViewReports;
use crate::viewport::capture::capture_in_flight::CaptureInFlight;
use crate::viewport::capture::pending_screenshot::PendingScreenshot;
use crate::viewport::pick::drawn_pick::DrawnPick;
use crate::viewport::pick::pending_pick::PendingPick;
use crate::viewport::pick::pick_in_flight::PickInFlight;
use crate::viewport::pick::pick_unprojection::PickUnprojection;
use crate::viewport::reported_graph::ReportedGraph;
use crate::viewport::viewport_device::ViewportDevice;
use crate::viewport::viewport_overlays::ViewportOverlays;
use crate::viewport::viewport_parts::ViewportParts;

/// One rectangle of a window drawn by the renderer, with what its world gave this frame and its page's channel. It owns
/// what its page asked of it: the captures, screenshots and picks waiting for a frame and on their way back, each taken
/// for a frame and put back where the frame could not draw it.
pub struct RenderViewport {
  id: RenderViewportId,
  /// The window it is drawn into.
  window: u64,
  /// Where the page last laid it out, or `None` before it has.
  layout: Option<RenderViewportLayout>,
  options: RenderViewOptions,
  overlays: ViewportOverlays,
  /// What of its level is selected, marked as it draws.
  selection: Option<RenderSelection>,
  /// The scene it shows and its view of it, made once a GPU is there; a scene of models alone keeps showing the one it
  /// replaces while its successor loads.
  shown: Option<ShownView>,
  /// The scene it asks to show, where its camera stands and what lights it this frame, as its world answered.
  world: RenderViewFrame,
  /// Captures asked for, copied out of the next frames presented, one a frame in the order asked, and those copied,
  /// answered once they are back.
  captures: Vec<CaptureReply>,
  /// Screenshots asked for, drawn from the next frames, one a frame in the order asked; in flight with the captures.
  screenshots: Vec<PendingScreenshot>,
  captures_in_flight: Vec<CaptureInFlight>,
  /// The level map or cubemap it captures, drawn in place of its own frames until it ends.
  capture: Option<CaptureJob>,
  /// The burst its own view draws, pacing the frames' clock until it ends.
  burst: Option<BurstJob>,
  /// Picks asked for, drawn one a frame while a readback is free, and those drawn, answered once they are back.
  picks: Vec<PendingPick>,
  picks_in_flight: Vec<PickInFlight>,
  /// Frames drawn, which a pick's or capture's answer names its own by.
  frame: u64,
  /// Its camera on the GPU, made once a GPU is there.
  binding: Option<ViewBinding>,
  sink: Box<dyn RenderEventSink>,
  statistics: FrameStatistics,
  /// What it was last told its frames cost, answered to a caller polling rather than listening.
  sent_frame: Option<RenderFrameReport>,
  /// What it was last told its frames are drawn with, and how far its scene has loaded.
  sent_applied: Option<RenderAppliedReport>,
  sent_load: Option<RenderLoadReport>,
  /// Whether its page stopped listening, after which it is detached.
  is_gone: bool,
  /// Whether it was told the renderer cannot draw, so it is told once.
  is_failed: bool,
}

/// The scene a viewport shows, and its own view of it.
struct ShownView {
  scene: RenderSceneId,
  view: SceneView,
}

impl RenderViewport {
  pub fn get_id(&self) -> RenderViewportId {
    self.id
  }

  /// The window it is drawn into.
  pub fn get_window(&self) -> u64 {
    self.window
  }

  /// Where the page last laid it out, none before it has.
  pub fn get_layout(&self) -> Option<&RenderViewportLayout> {
    self.layout.as_ref()
  }

  pub fn lay_out(&mut self, layout: RenderViewportLayout) {
    self.layout = Some(layout);
  }

  pub fn get_options(&self) -> &RenderViewOptions {
    &self.options
  }

  pub fn set_options(&mut self, options: RenderViewOptions) {
    self.options = options;
  }

  /// Takes what it draws over its frame: from now on for no scene, else from the frame it shows `scene`.
  pub fn set_overlays(&mut self, overlays: Vec<RenderOverlay>, scene: Option<RenderSceneId>) {
    let shown: Option<RenderSceneId> = self.get_shown_scene();

    self.overlays.set(overlays, scene, shown);
  }

  /// Takes what of its level is selected from now on, none for nothing.
  pub fn select(&mut self, selection: Option<RenderSelection>) {
    self.selection = selection;
  }

  /// The scene it asks to show, where its camera stands and what lights it this frame, as its world answered.
  pub fn get_world(&self) -> &RenderViewFrame {
    &self.world
  }

  /// Takes what its world answered for this frame.
  pub fn follow_world(&mut self, world: RenderViewFrame) {
    self.world = world;
  }

  /// The scene it shows, none where it shows none.
  pub fn get_shown_scene(&self) -> Option<RenderSceneId> {
    self.shown.as_ref().map(|shown| shown.scene)
  }

  /// Its own view of the scene it shows, whether or not it captures.
  pub fn get_shown_view(&self) -> Option<&SceneView> {
    self.shown.as_ref().map(|shown| &shown.view)
  }

  pub fn get_shown_view_mut(&mut self) -> Option<&mut SceneView> {
    self.shown.as_mut().map(|shown| &mut shown.view)
  }

  /// Its camera on the GPU, once a frame made it.
  pub fn get_binding(&self) -> Option<&ViewBinding> {
    self.binding.as_ref()
  }

  /// Lets its camera on the GPU go with the GPU, made again by the next GPU's first frame.
  pub fn forget_binding(&mut self) {
    self.binding = None;
  }

  /// Takes it apart for readying its frame: what it is drawn with, and its own view, binding and capture to write.
  pub fn split(&mut self) -> ViewportParts<'_> {
    let (scene, view): (Option<RenderSceneId>, Option<&mut SceneView>) = match self.shown.as_mut() {
      Some(shown) => (Some(shown.scene), Some(&mut shown.view)),
      None => (None, None),
    };

    ViewportParts {
      options: &self.options,
      world: &self.world,
      overlays: &self.overlays,
      selection: self.selection.as_ref(),
      scene,
      view,
      binding: &mut self.binding,
      capture: self.capture.as_mut().filter(|capture| !capture.is_ended()),
    }
  }

  /// Frames drawn, which a pick's or capture's answer names its own by.
  pub fn get_frame(&self) -> u64 {
    self.frame
  }

  /// Asks for a capture of its next presented frame.
  pub fn ask_capture(&mut self, reply: CaptureReply) {
    self.captures.push(reply);
  }

  /// Takes the first capture asked for, for this frame.
  pub fn take_capture(&mut self) -> Option<CaptureReply> {
    (!self.captures.is_empty()).then(|| self.captures.remove(0))
  }

  /// Puts a capture taken for a frame that could not copy it back first, for its next presented frame.
  pub fn return_capture(&mut self, reply: CaptureReply) {
    self.captures.insert(0, reply);
  }

  /// Asks for a screenshot drawn from its next frame.
  pub fn ask_screenshot(&mut self, screenshot: PendingScreenshot) {
    self.screenshots.push(screenshot);
  }

  /// Takes the first screenshot asked for, for this frame.
  pub fn take_screenshot(&mut self) -> Option<PendingScreenshot> {
    (!self.screenshots.is_empty()).then(|| self.screenshots.remove(0))
  }

  /// Keeps a capture or screenshot copied out of this frame until it is back.
  pub fn await_capture(&mut self, capture: FrameCapture, reply: CaptureReply) {
    self.captures_in_flight.push(CaptureInFlight { capture, reply });
  }

  /// Asks for a pick of its next drawn frame; one asked while it shows no scene meets nothing at once.
  pub fn ask_pick(&mut self, pick: PendingPick) {
    if self.shown.is_some() {
      self.picks.push(pick);
    } else {
      let _ = pick.reply.send(Ok(RenderPick {
        frame: self.frame,
        hit: None,
      }));
    }
  }

  /// Takes the first pick asked for, for this frame to draw.
  pub fn take_pick(&mut self) -> Option<PendingPick> {
    (!self.picks.is_empty()).then(|| self.picks.remove(0))
  }

  /// Puts a pick taken for a frame that did not draw it back first, for the next frame.
  pub fn return_pick(&mut self, pick: PendingPick) {
    self.picks.insert(0, pick);
  }

  /// Keeps a pick this frame drew into the readback `slot` until it is back.
  pub fn await_pick(&mut self, slot: usize, DrawnPick { pick, unprojection }: DrawnPick) {
    self.picks_in_flight.push(PickInFlight {
      pick,
      slot,
      unprojection,
      frame: self.frame,
    });
  }

  /// The burst its own view draws, while one runs.
  pub fn get_burst(&self) -> Option<&BurstJob> {
    self.burst.as_ref().filter(|burst| !burst.is_ended())
  }

  pub fn get_burst_mut(&mut self) -> Option<&mut BurstJob> {
    self.burst.as_mut().filter(|burst| !burst.is_ended())
  }

  /// Starts a burst its own view draws.
  pub fn start_burst(&mut self, burst: BurstJob) {
    self.burst = Some(burst);
  }

  /// The level map or cubemap it captures, while one runs.
  pub fn get_capture(&self) -> Option<&CaptureJob> {
    self.capture.as_ref().filter(|capture| !capture.is_ended())
  }

  pub fn get_capture_mut(&mut self) -> Option<&mut CaptureJob> {
    self.capture.as_mut().filter(|capture| !capture.is_ended())
  }

  /// Starts a capture drawn in place of its own frames.
  pub fn start_capture(&mut self, capture: CaptureJob) {
    self.capture = Some(capture);
  }

  /// Answers the picks and captures whose readbacks a poll found back, keeping those still on their way; a pick names
  /// what it hit in the scene it shows, among `scenes`.
  pub fn answer_readbacks(&mut self, scenes: &HashMap<RenderSceneId, LevelScene>) {
    let in_flight: Vec<PickInFlight> = std::mem::take(&mut self.picks_in_flight);

    for entry in in_flight {
      let Some((scene, level)) = self
        .shown
        .as_ref()
        .and_then(|shown| Some((scenes.get(&shown.scene)?, &shown.view)))
      else {
        let _ = entry.pick.reply.send(Ok(RenderPick {
          frame: entry.frame,
          hit: None,
        }));

        continue;
      };

      match level.take_pick(entry.slot) {
        Some(Ok(texel)) => {
          let PickUnprojection { view, ndc } = entry.unprojection;
          let inverse: Mat4 = view.get_view_projection().inverse();
          let hit: Option<RenderLevelHit> = level.resolve_pick(scene, texel, |stored: f32| -> (Vec3, f32) {
            let point: Vec3 = inverse.project_point3(ndc.extend(stored));

            (point, -view.view.transform_point3(point).z)
          });

          let _ = entry.pick.reply.send(Ok(RenderPick {
            frame: entry.frame,
            hit,
          }));
        }
        Some(Err(error)) => {
          let _ = entry.pick.reply.send(Err(error));
        }
        None => self.picks_in_flight.push(entry),
      }
    }

    let captures: Vec<CaptureInFlight> = std::mem::take(&mut self.captures_in_flight);

    for entry in captures {
      match entry.capture.take() {
        Some(result) => {
          let _ = entry.reply.send(result);
        }
        None => self.captures_in_flight.push(entry),
      }
    }

    if let Some(capture) = self.capture.as_mut() {
      capture.poll();
    }

    if self.capture.as_ref().is_some_and(CaptureJob::is_ended) {
      self.capture = None;
    }

    if let Some(burst) = self.burst.as_mut() {
      burst.poll();
    }

    if self.burst.as_ref().is_some_and(BurstJob::is_ended) {
      self.burst = None;
    }
  }

  /// The view it draws and its camera's binding: its capture's while it captures, else its own.
  pub fn get_drawn_view(&self) -> Option<(RenderSceneId, &SceneView, &ViewBinding)> {
    let shown: &ShownView = self.shown.as_ref()?;

    match self.get_capture() {
      Some(capture) => capture.get_view().map(|(view, binding)| (shown.scene, view, binding)),
      None => Some((shown.scene, &shown.view, self.binding.as_ref()?)),
    }
  }

  /// The view it draws: its capture's while it captures, else its own.
  pub fn get_drawn_view_mut(&mut self) -> Option<&mut SceneView> {
    let RenderViewport { shown, capture, .. } = self;
    let own: &mut SceneView = &mut shown.as_mut()?.view;

    match capture.as_mut().filter(|capture| !capture.is_ended()) {
      Some(capture) => capture.get_scene_view_mut(),
      None => Some(own),
    }
  }

  /// Whether it captures a level map or cubemap, drawn in place of its own frames.
  pub fn is_capturing(&self) -> bool {
    self.get_capture().is_some()
  }

  /// Stops what it captures, telling its caller why.
  pub fn stop_capture(&mut self, reason: &str) {
    if let Some(mut capture) = self.capture.take() {
      capture.stop(XrfError::new_invalid_error(reason.to_string()));
    }
  }

  /// Whether its own view draws a burst, whose frames are drawn whatever its window shows.
  pub fn is_bursting(&self) -> bool {
    self.get_burst().is_some()
  }

  /// Stops the burst it draws, telling its caller why.
  pub fn stop_burst(&mut self, reason: &str) {
    if let Some(mut burst) = self.burst.take() {
      burst.stop(XrfError::new_invalid_error(reason.to_string()));
    }
  }

  /// Answers every capture in flight as its GPU goes: what was read back, or a refusal where nothing came back, as on a
  /// lost device, whose readbacks may never finish.
  pub fn release_captures(&mut self) {
    self.stop_capture("The GPU went before the capture was read back");
    self.stop_burst("The GPU went before the burst was read back");

    for CaptureInFlight { capture, reply } in std::mem::take(&mut self.captures_in_flight) {
      let _ = reply.send(capture.take().unwrap_or_else(|| {
        Err(XrfError::new_unexpected_error(
          "The GPU went before the capture was read back",
        ))
      }));
    }
  }

  /// Shows the scene `scene` through `view`, a view of it of its own, with the overlays given for it.
  pub fn show(&mut self, scene: RenderSceneId, view: SceneView) {
    self.leave_shown();
    self.shown = Some(ShownView { scene, view });
    self.overlays.show(scene);
  }

  /// Shows no scene.
  pub fn hide(&mut self) {
    self.leave_shown();
    self.shown = None;
  }

  /// Leaves the view it showed: the picks asked of it, drawn or still waiting, meet nothing, since what they were
  /// asked of is drawn no more, and what it captures stops.
  fn leave_shown(&mut self) {
    self.stop_capture("The viewport shows another scene than the capture was asked of");
    self.stop_burst("The viewport shows another scene than the burst was asked of");

    for entry in std::mem::take(&mut self.picks_in_flight) {
      let _ = entry.pick.reply.send(Ok(RenderPick {
        frame: entry.frame,
        hit: None,
      }));
    }

    self.answer_picks_unseen();
  }

  /// Answers the picks waiting to be drawn as meeting nothing, for a frame in which it was not drawn.
  pub fn answer_picks_unseen(&mut self) {
    for pick in std::mem::take(&mut self.picks) {
      let _ = pick.reply.send(Ok(RenderPick {
        frame: self.frame,
        hit: None,
      }));
    }
  }

  pub fn new(id: RenderViewportId, window: u64, sink: Box<dyn RenderEventSink>, now: Instant) -> Self {
    Self {
      id,
      window,
      layout: None,
      options: RenderViewOptions::default(),
      overlays: ViewportOverlays::default(),
      selection: None,
      shown: None,
      world: RenderViewFrame::default(),
      captures: Vec::new(),
      screenshots: Vec::new(),
      captures_in_flight: Vec::new(),
      capture: None,
      burst: None,
      picks: Vec::new(),
      picks_in_flight: Vec::new(),
      frame: 0,
      binding: None,
      sink,
      statistics: FrameStatistics::new(now),
      sent_frame: None,
      sent_applied: None,
      sent_load: None,
      is_gone: false,
      is_failed: false,
    }
  }

  /// Where it is drawn in a window frame of the given size, or `None` when nothing of it shows.
  pub fn get_drawn_rect(&self, width: u32, height: u32) -> Option<RenderRect> {
    self.layout.as_ref().and_then(|layout| layout.rect.clip(width, height))
  }

  /// Device pixels per CSS pixel.
  pub fn get_scale(&self) -> f32 {
    self.layout.as_ref().map_or(1.0, |layout| layout.scale.max(0.1))
  }

  /// The viewport's height in CSS pixels, which drags are measured in.
  pub fn get_css_height(&self) -> f32 {
    self
      .layout
      .as_ref()
      .map_or(1.0, |layout| layout.rect.height as f32 / layout.scale.max(0.1))
  }

  /// What its frames cost when it last reported them, none before its first report.
  pub fn get_frame_report(&self) -> Option<&RenderFrameReport> {
    self.sent_frame.as_ref()
  }

  pub fn is_gone(&self) -> bool {
    self.is_gone
  }

  /// Whose its passes are in a frame's graph: a part of its own, which its number keeps apart from the frame's.
  pub fn get_graph_owner(&self) -> GraphOwner {
    GraphOwner::Part(self.id.0)
  }

  /// Counts a frame presented and notes it for the next report: its `interval`, its time on the render thread (`cpu`),
  /// the GPU's latest span, phases and encode groups.
  pub fn count_frame(
    &mut self,
    interval: Duration,
    cpu: Duration,
    gpu: Option<f32>,
    phases: &FramePhases,
    groups: &[ExecutedGroup],
  ) {
    self.frame += 1;
    self.statistics.record(interval, cpu, gpu, phases, groups);
  }

  /// Reports the frames since the last report, once one is due: `allocator` what the GPU allocator and the GPU's own
  /// memory hold, `scenes` every scene, its own among them, `timer` what the frames' passes cost, its own taken from
  /// it, `composition` what the windows' own passes cost this report round, and `graph` what the frame graph made of
  /// the latest described frame.
  pub fn report(
    &mut self,
    now: Instant,
    device: ViewportDevice<'_>,
    allocator: Option<&GpuMemory>,
    scenes: &mut HashMap<RenderSceneId, LevelScene>,
    ReportedGraph {
      timer,
      composition,
      graph,
    }: ReportedGraph<'_>,
  ) {
    let Some(summary) = self.statistics.take(now) else {
      return;
    };
    let ViewportDevice {
      backend,
      adapter,
      has_ray_queries,
      dlss,
      ray_reconstruction,
    } = device;
    let shown: Option<(&mut LevelScene, &mut SceneView)> = self
      .shown
      .as_mut()
      .and_then(|shown| Some((scenes.get_mut(&shown.scene)?, &mut shown.view)));
    let (reports, sector_time): (Option<ViewReports>, f32) = match shown {
      Some((scene, view)) => {
        let is_driving: bool = scene.get_driver() == Some(self.id);

        (
          Some(view.take_reports(scene, &self.options, is_driving)),
          scene.get_sector_time(),
        )
      }
      None => (None, 0.0),
    };
    let mut times: Vec<GraphPassTime> = timer.take(self.get_graph_owner());

    times.extend_from_slice(composition);

    let is_gpu_timed: bool = timer.is_timing();
    let passes: Vec<RenderPassCost> = times
      .into_iter()
      .map(|time| RenderPassCost {
        name: time.name,
        gpu_time: time.gpu_time,
      })
      .collect();
    let FrameSummary {
      frames_per_second,
      frame_time,
      frame_time_max,
      frame_times,
      cpu_time,
      cpu_times,
      gpu_time,
      gpu_times,
      phases,
      groups,
    } = summary;
    let rect: RenderRect = self.layout.as_ref().map(|layout| layout.rect).unwrap_or_default();
    // What its frames are drawn with is told the page only while a view is shown.
    let is_shown: bool = reports.is_some();
    let ViewReports {
      static_draws,
      lights,
      particles,
      drawables,
      meter,
      render_size,
      output_size,
      antialiasing,
      is_ray_reconstructed,
      applied,
    } = reports.unwrap_or_default();
    let (render_width, render_height): (u32, u32) = render_size.unwrap_or((rect.width, rect.height));
    let (output_width, output_height): (u32, u32) = output_size.unwrap_or((rect.width, rect.height));
    let report: RenderFrameReport = RenderFrameReport {
      frames_per_second,
      frame_time,
      frame_time_max,
      frame_times,
      cpu_time,
      cpu_times,
      gpu_time,
      gpu_times,
      phases,
      groups,
      width: rect.width,
      height: rect.height,
      render_width,
      render_height,
      output_width,
      output_height,
      antialiasing,
      is_ray_reconstructed,
      backend,
      adapter: adapter.to_string(),
      has_ray_queries,
      dlss,
      ray_reconstruction,
      is_gpu_timed,
      passes,
      static_draws,
      lights,
      particles,
      drawables,
      sector_time,
      graph: graph.map(|graph| Box::new(RenderGraphReport::of(graph, self.get_graph_owner()))),
      memory: RenderMemoryReport::of(allocator),
      meter,
    };

    self.sent_frame = Some(report.clone());
    self.send(RenderViewportEvent::Frame {
      report: Box::new(report),
    });

    if is_shown {
      self.publish_applied(applied);
    }
  }

  /// Tells the page what its frames are drawn with (`applied`, the weather's light added here), where that changed
  /// since it was last told.
  fn publish_applied(&mut self, mut applied: RenderAppliedReport) {
    if self.options.asset.lighting.is_none() {
      applied.environment = Some(Self::to_applied_environment(
        &self.world.lighting,
        &self.options.features.light_scales,
      ));
    }

    if self.sent_applied.as_ref() != Some(&applied) {
      self.sent_applied = Some(applied.clone());
      self.send(RenderViewportEvent::Applied { report: applied });
    }
  }

  /// Whether its next report is due.
  pub fn is_report_due(&self, now: Instant) -> bool {
    self.statistics.is_due(now)
  }

  /// Tells the page how far the scene it shows has loaded, where that changed since it was last told.
  pub fn report_load(&mut self, report: RenderLoadReport) {
    if self.sent_load != Some(report) {
      self.sent_load = Some(report);
      self.send(RenderViewportEvent::Load { report });
    }
  }

  /// Tells the page the renderer cannot draw, once until it can again.
  pub fn fail(&mut self, message: &str) {
    if !self.is_failed {
      self.is_failed = true;
      self.send(RenderViewportEvent::Failure {
        message: message.to_string(),
      });
    }
  }

  /// Lets a later failure be told again, once the renderer draws again.
  pub fn recover(&mut self) {
    self.is_failed = false;
  }

  fn send(&mut self, event: RenderViewportEvent) {
    if !self.sink.send(event) {
      self.is_gone = true;
    }
  }

  /// What `lighting` lights a scene with, as the passes bind it.
  fn to_applied_environment(lighting: &RenderLighting, scales: &RenderLightScales) -> RenderAppliedEnvironment {
    RenderAppliedEnvironment {
      sun_direction: lighting.get_sun_direction().to_array(),
      sun_color: lighting.get_sun_color(scales).to_array(),
      ambient: lighting.get_ambient(scales).to_array(),
      hemisphere: lighting.get_environment(scales).to_array(),
      fog: lighting.fog.as_ref().map(|fog| RenderAppliedFog {
        color: fog.color.to_array(),
        distance: fog.get_total_distance(),
        density: fog.density,
      }),
      rain_density: lighting.rain.as_ref().map_or(0.0, |rain| rain.density),
      tree_sway: lighting.trees.as_ref().map_or(0.0, |trees| trees.amplitude),
      water_intensity: lighting.water_intensity,
    }
  }
}
