use std::collections::HashMap;
use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_renderer_core::{FrameGraph, GraphPassTime, GraphReport, GraphTimer};

use crate::contract::render_applied_environment::RenderAppliedEnvironment;
use crate::contract::render_applied_fog::RenderAppliedFog;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_graph_report::RenderGraphReport;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_light_scales::RenderLightScales;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_memory_report::RenderMemoryReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pass_cost::RenderPassCost;
use crate::contract::render_pick::RenderPick;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_viewport_event::RenderViewportEvent;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::frame::frame_capture::{CaptureReply, FrameCapture};
use crate::frame::frame_phases::FramePhases;
use crate::frame::frame_statistics::{FrameStatistics, FrameSummary};
use crate::host::render_event_sink::RenderEventSink;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_view_frame::RenderViewFrame;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::scene_view::SceneView;
use crate::viewport::pending_pick::PendingPick;
use crate::viewport::pick_in_flight::PickInFlight;

/// One rectangle of a window drawn by the renderer, with what its world gave this frame and its page's channel.
pub struct RenderViewport {
  pub id: RenderViewportId,
  /// The window it is drawn into.
  pub window: u64,
  /// Where the page last laid it out, or `None` before it has.
  pub layout: Option<RenderViewportLayout>,
  pub options: RenderViewOptions,
  /// What it draws over its frame, and how many sets it has been given, which its level's vertices follow.
  pub overlays: Vec<RenderOverlay>,
  pub overlays_version: u64,
  /// What of its level is selected, marked as it draws.
  pub selection: Option<RenderSelection>,
  /// The scene it shows and its view of it, made once a GPU is there; a scene of models alone keeps showing the one it
  /// replaces while its successor loads.
  pub shown: Option<(RenderSceneId, SceneView)>,
  /// The scene it asks to show, where its camera stands and what lights it this frame, as its world answered.
  pub world: RenderViewFrame,
  /// Captures asked for, copied out of the next frame presented, and those copied, answered once they are back.
  pub captures: Vec<CaptureReply>,
  pub captures_in_flight: Vec<(FrameCapture, CaptureReply)>,
  /// Picks asked for, drawn one a frame while a readback is free, and those drawn, answered once they are back.
  pub picks: Vec<PendingPick>,
  pub picks_in_flight: Vec<PickInFlight>,
  /// Frames drawn, which a pick's or capture's answer names its own by.
  pub frame: u64,
  /// Its camera on the GPU, made once a GPU is there.
  pub binding: Option<ViewBinding>,
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

impl RenderViewport {
  /// Answers the picks and captures whose readbacks a poll found back, keeping those still on their way; a pick names
  /// what it hit in the scene it shows, among `scenes`.
  pub fn answer_readbacks(&mut self, scenes: &HashMap<RenderSceneId, LevelScene>) {
    let in_flight: Vec<PickInFlight> = std::mem::take(&mut self.picks_in_flight);

    for entry in in_flight {
      let Some((scene, level)) = self.shown.as_ref().and_then(|(id, view)| Some((scenes.get(id)?, view))) else {
        let _ = entry.pick.reply.send(Ok(RenderPick {
          frame: entry.frame,
          hit: None,
        }));

        continue;
      };

      match level.take_pick(entry.slot) {
        Some(Ok(texel)) => {
          let (inverse, ndc) = entry.unprojection;
          let hit: Option<RenderLevelHit> = level.resolve_pick(scene, texel, |depth: f32| -> Vec3 {
            inverse.project_point3(ndc.extend(depth))
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

    let captures: Vec<(FrameCapture, CaptureReply)> = std::mem::take(&mut self.captures_in_flight);

    for (capture, reply) in captures {
      match capture.take() {
        Some(result) => {
          let _ = reply.send(result);
        }
        None => self.captures_in_flight.push((capture, reply)),
      }
    }
  }

  pub fn new(id: RenderViewportId, window: u64, sink: Box<dyn RenderEventSink>, now: Instant) -> Self {
    Self {
      id,
      window,
      layout: None,
      options: RenderViewOptions::default(),
      overlays: Vec::new(),
      overlays_version: 0,
      selection: None,
      shown: None,
      world: RenderViewFrame::default(),
      captures: Vec::new(),
      captures_in_flight: Vec::new(),
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

  /// Reports the frames since the last report, once one is due: `texture_bytes` is what every viewport's textures hold
  /// on the GPU together, `scenes` every scene, its own among them, `timer` what the frames' passes cost, its own and
  /// its window's taken from it, and `graph` what the frame graph made of the latest described frame.
  pub fn report(
    &mut self,
    now: Instant,
    (backend, adapter): (RenderBackend, &str),
    (texture_bytes, scenes): (u64, &mut HashMap<RenderSceneId, LevelScene>),
    (timer, graph): (Option<&mut GraphTimer>, Option<&GraphReport>),
  ) {
    let Some(summary) = self.statistics.take(now) else {
      return;
    };
    let shown: Option<(&mut LevelScene, &mut SceneView)> = self
      .shown
      .as_mut()
      .and_then(|(id, view)| Some((scenes.get_mut(id)?, view)));
    let (static_draws, lights, particles, sector_time): (
      RenderStaticReport,
      RenderLightsReport,
      RenderParticlesReport,
      f32,
    ) = match shown {
      Some((scene, view)) => {
        let (static_draws, lights) = view.take_stats(scene);

        (
          static_draws,
          lights,
          view.take_particles_report(scene),
          scene.sector_time,
        )
      }
      None => Default::default(),
    };
    let (is_gpu_timed, passes): (bool, Vec<RenderPassCost>) = timer.map_or((false, Vec::new()), |timer| {
      let mut times: Vec<GraphPassTime> = timer.take(self.id.0);

      times.extend(timer.take(FrameGraph::FRAME_OWNER));

      (
        timer.is_timing(),
        times
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
      .shown
      .as_ref()
      .and_then(|(_, view)| view.get_state().get_render_size())
      .unwrap_or((rect.width, rect.height));
    // The scene it shows, and the one it asks for while that comes in.
    let mut held: Vec<RenderSceneId> = self.shown.iter().map(|(id, _)| *id).chain(self.world.scene).collect();

    held.dedup();

    let scene: u64 = held
      .iter()
      .filter_map(|id| scenes.get(id))
      .map(LevelScene::get_buffer_bytes)
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
      backend,
      adapter: adapter.to_string(),
      is_gpu_timed,
      passes,
      static_draws,
      lights,
      particles,
      sector_time,
      graph: graph.map(|graph| Box::new(RenderGraphReport::of(graph, self.id.0, FrameGraph::FRAME_OWNER))),
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
    let Some((_, level)) = self.shown.as_ref() else {
      return;
    };
    let mut applied: RenderAppliedReport = level.describe_applied(&self.options);

    if self.options.asset.lighting.is_none() {
      applied.environment = Some(to_applied_environment(
        &self.world.lighting,
        &self.options.features.light_scales,
      ));
    }

    if self.sent_applied.as_ref() != Some(&applied) {
      self.sent_applied = Some(applied.clone());
      self.send(RenderViewportEvent::Applied { report: applied });
    }
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

  pub fn recover(&mut self) {
    self.is_failed = false;
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
