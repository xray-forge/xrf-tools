use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_renderer_core::GraphRuntime;

use crate::camera::camera_controller::CameraController;
use crate::contract::render_ambient_report::RenderAmbientReport;
use crate::contract::render_applied_environment::RenderAppliedEnvironment;
use crate::contract::render_applied_fog::RenderAppliedFog;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_camera_pose::RenderCameraPose;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_light_scales::RenderLightScales;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_memory_report::RenderMemoryReport;
use crate::contract::render_model_pose::RenderModelPose;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pass_cost::RenderPassCost;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_event::RenderViewportEvent;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::frame::frame_capture::CaptureReply;
use crate::frame::frame_phases::FramePhases;
use crate::frame::frame_statistics::{FrameStatistics, FrameSummary};
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_level_source::RenderLevelSource;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_view::LevelView;
use crate::thread::render_workers::RenderWorkers;
use crate::viewport::pending_pick::PendingPick;
use crate::weather::viewport_weather::ViewportWeather;

/// How often a moving camera's pose is published.
const POSE_INTERVAL: Duration = Duration::from_millis(100);

/// One rectangle of a window drawn by the renderer, with its camera and its page's channel.
pub struct RenderViewport {
  pub id: RenderViewportId,
  /// The window it is drawn into.
  pub window: u64,
  /// Where the page last laid it out, or `None` before it has.
  pub layout: Option<RenderViewportLayout>,
  pub camera: CameraController,
  pub options: RenderViewOptions,
  /// What it draws over its frame, and how many sets it has been given, which its level's vertices follow.
  pub overlays: Vec<RenderOverlay>,
  pub overlays_version: u64,
  /// What of its level is selected, marked as it draws.
  pub selection: Option<RenderSelection>,
  /// How its skinned models stand.
  pub model_pose: RenderModelPose,
  /// The level it draws, as its source gives it.
  pub level: Option<Arc<dyn RenderLevelSource>>,
  /// The level as this viewport draws it, made once a GPU is there; a scene of models alone keeps the one it replaces
  /// here while its successor loads.
  pub level_view: Option<LevelView>,
  /// The successor of a scene of models alone, loading out of sight until it can be drawn whole.
  pub incoming_view: Option<LevelView>,
  /// The level's weather, which lights it.
  pub weather: ViewportWeather,
  /// Captures asked for, answered by the next frame presented.
  pub captures: Vec<CaptureReply>,
  /// Picks asked for, answered one a frame.
  pub picks: Vec<PendingPick>,
  /// Its camera on the GPU, made once a GPU is there.
  pub binding: Option<ViewBinding>,
  /// What its frame graphs keep between frames, its passes' GPU timer among them; made once a GPU is there, and kept
  /// across levels.
  pub runtime: Option<GraphRuntime>,
  sink: Box<dyn RenderEventSink>,
  statistics: FrameStatistics,
  /// What it was last told its frames cost, answered to a caller polling rather than listening.
  sent_frame: Option<RenderFrameReport>,
  sent_pose: Option<RenderCameraPose>,
  /// What it was last told its frames are drawn with.
  sent_applied: Option<RenderAppliedReport>,
  pose_due: Instant,
  /// Whether its page stopped listening, after which it is detached.
  is_gone: bool,
  /// Whether it was told the renderer cannot draw, so it is told once.
  is_failed: bool,
}

impl RenderViewport {
  pub fn new(
    id: RenderViewportId,
    window: u64,
    sink: Box<dyn RenderEventSink>,
    now: Instant,
    workers: &RenderWorkers,
  ) -> Self {
    Self {
      id,
      window,
      layout: None,
      camera: CameraController::default(),
      options: RenderViewOptions::default(),
      overlays: Vec::new(),
      overlays_version: 0,
      selection: None,
      model_pose: RenderModelPose::default(),
      level: None,
      level_view: None,
      incoming_view: None,
      weather: ViewportWeather::new(now, workers),
      captures: Vec::new(),
      picks: Vec::new(),
      binding: None,
      runtime: None,
      sink,
      statistics: FrameStatistics::new(now),
      sent_frame: None,
      sent_pose: None,
      sent_applied: None,
      pose_due: now,
      is_gone: false,
      is_failed: false,
    }
  }

  /// Where it is drawn in a window frame of the given size, or `None` when nothing of it shows.
  pub fn get_drawn_rect(&self, width: u32, height: u32) -> Option<RenderRect> {
    self.layout.and_then(|layout| layout.rect.clip(width, height))
  }

  /// Device pixels per CSS pixel.
  pub fn get_scale(&self) -> f32 {
    self.layout.map_or(1.0, |layout| layout.scale.max(0.1))
  }

  /// The viewport's height in CSS pixels, which drags are measured in.
  pub fn get_css_height(&self) -> f32 {
    self
      .layout
      .map_or(1.0, |layout| layout.rect.height as f32 / layout.scale.max(0.1))
  }

  /// The view of the level it was last asked to show: the one coming in while it loads, else the one drawn; none before
  /// that level's view is made, or where it shows no level.
  pub fn get_asked_view(&self) -> Option<&LevelView> {
    let source: &Arc<dyn RenderLevelSource> = self.level.as_ref()?;

    [&self.incoming_view, &self.level_view]
      .into_iter()
      .flatten()
      .find(|view| view.get_scene().is_showing(source))
  }

  /// What its frames cost when it last reported them, none before its first report.
  pub fn get_frame_report(&self) -> Option<&RenderFrameReport> {
    self.sent_frame.as_ref()
  }

  pub fn is_gone(&self) -> bool {
    self.is_gone
  }

  pub fn record_frame(&mut self, interval: Duration, cpu: Duration, phases: &FramePhases) {
    self.statistics.record(interval, cpu, phases);
  }

  /// Reports the frames since the last report, once one is due.
  /// `texture_bytes` is what every viewport's textures hold on the GPU together.
  pub fn report(&mut self, now: Instant, backend: &str, adapter: &str, texture_bytes: u64) {
    let Some(summary) = self.statistics.take(now) else {
      return;
    };
    let (static_draws, lights): (RenderStaticReport, RenderLightsReport) = self
      .level_view
      .as_mut()
      .map_or_else(Default::default, |level| level.take_stats());
    let particles: RenderParticlesReport = self
      .level_view
      .as_mut()
      .map_or_else(Default::default, |level| level.get_scene_mut().take_particles_report());
    let (is_gpu_timed, passes): (bool, Vec<RenderPassCost>) =
      self.runtime.as_mut().map_or((false, Vec::new()), |runtime| {
        (
          runtime.timer.is_timing(),
          runtime
            .timer
            .take()
            .into_iter()
            .map(|time| RenderPassCost {
              name: time.name,
              gpu_time: time.gpu_time,
            })
            .collect(),
        )
      });
    let FrameSummary {
      frames_per_second,
      frame_time,
      frame_time_max,
      cpu_time,
      phases,
    } = summary;
    let rect: RenderRect = self.layout.map(|layout| layout.rect).unwrap_or_default();
    let (render_width, render_height): (u32, u32) = self
      .level_view
      .as_ref()
      .and_then(|level| level.get_state().get_render_size())
      .unwrap_or((rect.width, rect.height));

    let scene: u64 = [&self.level_view, &self.incoming_view]
      .into_iter()
      .flatten()
      .map(|level| level.get_scene().get_buffer_bytes())
      .sum();

    let report: RenderFrameReport = RenderFrameReport {
      frames_per_second,
      frame_time,
      frame_time_max,
      cpu_time,
      phases,
      width: rect.width,
      height: rect.height,
      render_width,
      render_height,
      backend: backend.to_string(),
      adapter: adapter.to_string(),
      is_gpu_timed,
      passes,
      static_draws,
      lights,
      particles,
      sector_time: self
        .level_view
        .as_ref()
        .map_or(0.0, |level| level.get_scene().get_sector_time()),
      memory: RenderMemoryReport {
        textures: texture_bytes,
        scene,
      },
    };

    self.sent_frame = Some(report.clone());
    self.send(RenderViewportEvent::Frame { report });
    self.publish_applied();
  }

  /// Tells the page what its frames are drawn with, where that changed since it was last told.
  fn publish_applied(&mut self) {
    let Some(level) = self.level_view.as_ref() else {
      return;
    };
    let mut applied: RenderAppliedReport = level.describe_applied(&self.options);

    if self.options.asset_lighting.is_none() {
      applied.environment = Some(to_applied_environment(
        self.weather.get_lighting(),
        &self.options.light_scales,
      ));
    }

    if self.sent_applied.as_ref() != Some(&applied) {
      self.sent_applied = Some(applied.clone());
      self.send(RenderViewportEvent::Applied { report: applied });
    }
  }

  /// Publishes the camera's pose when it changed, at most every [`POSE_INTERVAL`]; the last change of a motion is
  /// published at the next interval after it stops.
  pub fn publish_pose(&mut self, now: Instant) {
    if now < self.pose_due {
      return;
    }

    let pose: RenderCameraPose = self.camera.get_pose();

    if self.sent_pose != Some(pose) {
      self.sent_pose = Some(pose);
      self.pose_due = now + POSE_INTERVAL;
      self.send(RenderViewportEvent::Camera { pose });
    }
  }

  /// Publishes where the weather and its ambient effects stand when they changed, a few times a second at most.
  pub fn publish_weather(&mut self, now: Instant) {
    let ambient: Option<RenderAmbientReport> = self
      .level_view
      .as_ref()
      .and_then(|level| level.get_scene().get_ambient_report());

    if let Some(report) = self.weather.take_report(now, ambient) {
      self.send(RenderViewportEvent::Weather { report });
    }
  }

  pub fn report_load(&mut self, report: RenderLoadReport) {
    self.send(RenderViewportEvent::Load { report });
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

  pub fn recover(&mut self) {
    self.is_failed = false;
    // Its pool, bind groups and timer belong to the device that was lost.
    self.runtime = None;
  }

  fn send(&mut self, event: RenderViewportEvent) {
    if !self.sink.send(event) {
      self.is_gone = true;
    }
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
