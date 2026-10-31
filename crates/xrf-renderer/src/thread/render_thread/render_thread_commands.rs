use std::sync::PoisonError;
use std::sync::mpsc::Sender;
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};

use crate::capture::burst::burst_event::BurstEvent;
use crate::capture::burst::burst_job::BurstJob;
use crate::capture::capture_event::CaptureEvent;
use crate::capture::capture_job::CaptureJob;
use crate::capture::capture_plan::CapturePlan;
use crate::contract::render_backend::RenderBackend;
use crate::contract::render_burst::RenderBurst;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_memory_report::RenderMemoryReport;
use crate::contract::render_pick::RenderPick;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::host::render_scene_id::RenderSceneId;
use crate::scene::level::level_scene::LevelScene;
use crate::thread::render_command::RenderCommand;
use crate::thread::running_gpu::RunningGpu;
use crate::viewport::pending_pick::PendingPick;
use crate::viewport::render_viewport::RenderViewport;

use super::RenderThread;

impl RenderThread {
  pub(super) fn handle(&mut self, command: RenderCommand) {
    match command {
      RenderCommand::Attach { id, host, sink } => {
        let window: u64 = host.get_key();

        self.hosts.entry(window).or_insert(host);
        self
          .viewports
          .insert(id, RenderViewport::new(id, window, sink, Instant::now()));
        self.idle_since = None;
        // An attach after a stop keeps the thread drawing; the stop's answer goes unsent.
        self.stopping = None;
      }
      RenderCommand::Detach { id } => {
        self.viewports.remove(&id);
        self.forget_unused_windows();
      }
      RenderCommand::Handled { reply } => {
        let _ = reply.send(());
      }
      RenderCommand::Stop { reply } => {
        self.viewports.clear();
        self.forget_unused_windows();
        self.stopping = Some(reply);
      }
      RenderCommand::Layout { id, layout } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.layout = Some(layout);
        }
      }
      RenderCommand::ReloadTextures { names } => {
        if let Some(RunningGpu { gpu, .. }) = &mut self.running {
          gpu.textures.reload(&names);
          gpu.weather_textures.reload(&names);
        }
      }
      RenderCommand::Settings { settings } => {
        // A changed backend or GPU takes effect at once: a running GPU restarts unless it already matches the choice,
        // and a failed start retries without waiting. A change to the automatic GPU always restarts, since it may pick
        // a different one. A choice that cannot start falls back as at any start.
        let is_backend_changed: bool = settings.backend != self.settings.backend;
        let is_gpu_changed: bool = settings.gpu != self.settings.gpu;

        if is_backend_changed || is_gpu_changed {
          let preferred: RenderBackend = RenderBackend::get_preferred(RenderBackend::get_asked(settings.backend));
          let is_restarted: bool = self.running.as_ref().is_some_and(|RunningGpu { gpu, .. }| {
            (is_backend_changed && gpu.context.backend != preferred)
              || (is_gpu_changed && settings.gpu.is_none_or(|asked| asked != gpu.context.gpu))
          });

          if is_restarted {
            log::info!("Restarting the renderer on {}", preferred.get_label());
            self.release_gpu();
          }

          self.failure = None;
        }

        self.settings = settings;
      }
      RenderCommand::Options { id, options } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.options = *options;
        }
      }
      RenderCommand::Overlays { id, overlays } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.overlays = overlays;
          viewport.overlays_version += 1;
        }
      }
      RenderCommand::Selection { id, selection } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.selection = selection;
        }
      }
      RenderCommand::Pick {
        id,
        x,
        y,
        is_through,
        reply,
      } => {
        let pick: PendingPick = PendingPick {
          x,
          y,
          is_through,
          reply,
        };

        match self.viewports.get_mut(&id) {
          Some(viewport) if viewport.shown.is_some() => viewport.picks.push(pick),
          // Nothing is drawn there to pick.
          Some(viewport) => {
            let _ = pick.reply.send(Ok(RenderPick {
              frame: viewport.frame,
              hit: None,
            }));
          }
          None => {
            let _ = pick.reply.send(Ok(RenderPick { frame: 0, hit: None }));
          }
        }
      }
      RenderCommand::DescribeTextures { id, reply } => {
        let described: Vec<RenderTextureReport> = self
          .running
          .as_ref()
          .map(|running| &running.gpu)
          .zip(self.get_shown_scene(id))
          .map(|(gpu, scene)| scene.describe_textures(&gpu.textures))
          .unwrap_or_default();

        let _ = reply.send(described);
      }
      RenderCommand::DescribeLoad { id, reply } => {
        let described: Option<RenderLoadReport> = self
          .running
          .as_ref()
          .map(|running| &running.gpu)
          .zip(
            self
              .get_asked_scene(id)
              .and_then(|asked| Some((asked, self.scenes.get(&asked)?))),
          )
          .map(|(gpu, (asked, scene))| scene.describe_load(asked, &gpu.textures));

        let _ = reply.send(described);
      }
      RenderCommand::DescribeFrame { id, reply } => {
        let described: Option<RenderFrameReport> = self
          .viewports
          .get(&id)
          .and_then(RenderViewport::get_frame_report)
          .cloned();

        let _ = reply.send(described);
      }
      RenderCommand::DescribeMemory { reply } => {
        let described: Option<RenderMemoryReport> = self.running.as_ref().map(|running| {
          RenderMemoryReport::of(Self::read_allocator_report(
            &mut self.allocator_report,
            (&running.gpu.context.adapter, &running.gpu.context.device),
            Instant::now(),
          ))
        });

        let _ = reply.send(described);
      }
      RenderCommand::LocateSpawnObject { id, object, reply } => {
        let sphere: Option<[f32; 4]> = self
          .get_shown_scene(id)
          .and_then(|scene| scene.get_object_sphere(object))
          .map(|sphere| sphere.to_array());

        let _ = reply.send(sphere);
      }
      RenderCommand::Capture { id, reply } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.captures.push(reply);
        }
      }
      RenderCommand::Screenshot { id, is_helped, reply } => {
        if let Some(viewport) = self.viewports.get_mut(&id) {
          viewport.screenshots.push((reply, is_helped));
        }
      }
      RenderCommand::CaptureMap { id, capture, events } => {
        let most: u32 = self.get_most_texture();

        self.begin_capture(id, events, |_| CapturePlan::new_map(capture, most));
      }
      RenderCommand::CaptureCube { id, capture, events } => {
        let most: u32 = self.get_most_texture();

        self.begin_capture(id, events, |viewport| {
          CapturePlan::new_cube(capture, &viewport.world.camera, Self::get_far_limit(viewport), most)
        });
      }
      RenderCommand::CaptureBurst { id, burst, events } => self.begin_burst(id, burst, events),
      RenderCommand::StepClock { step } => self.clock.set_step(step),
    }
  }

  /// Starts a viewport's burst; refused, through `events`, where the viewport is gone, shows no level or captures, a
  /// burst already paces the clock, or the burst cannot be drawn.
  fn begin_burst(&mut self, id: RenderViewportId, burst: RenderBurst, events: Sender<BurstEvent>) {
    let refusal: Option<String> = match self.viewports.get(&id) {
      None => Some(format!("Viewport {} is no longer drawn", id.0)),
      Some(viewport) if viewport.shown.is_none() => Some(format!("Viewport {} shows no level to burst", id.0)),
      Some(viewport) if viewport.is_capturing() => Some(format!("Viewport {} captures already", id.0)),
      Some(_) if self.viewports.values().any(RenderViewport::is_bursting) => {
        Some(String::from("Another burst paces the frames' clock"))
      }
      Some(_) => None,
    };

    if let Some(refusal) = refusal {
      let _ = events.send(BurstEvent::Stopped {
        error: XrfError::new_invalid_error(refusal),
      });

      return;
    }

    match (BurstJob::new(burst, events.clone()), self.viewports.get_mut(&id)) {
      (Ok(job), Some(viewport)) => viewport.burst = Some(job),
      (Err(error), _) => {
        let _ = events.send(BurstEvent::Stopped { error });
      }
      (Ok(_), None) => {}
    }
  }

  /// The pixels on a side of the GPU's largest texture, or what every GPU draws before one is there.
  fn get_most_texture(&self) -> u32 {
    self
      .running
      .as_ref()
      .map_or(wgpu::Limits::default().max_texture_dimension_2d, |running| {
        running.gpu.context.device.limits().max_texture_dimension_2d
      })
  }

  /// Starts a viewport's capture as `plan` plans it from the viewport; refused, through `events`, where the viewport is
  /// gone, shows no level or captures already, or the capture cannot be drawn.
  fn begin_capture(
    &mut self,
    id: RenderViewportId,
    events: Sender<CaptureEvent>,
    plan: impl FnOnce(&RenderViewport) -> XrfResult<CapturePlan>,
  ) {
    let planned: XrfResult<CapturePlan> = match self.viewports.get(&id) {
      None => Err(XrfError::new_invalid_error(format!(
        "Viewport {} is no longer drawn",
        id.0
      ))),
      Some(viewport) if viewport.shown.is_none() => Err(XrfError::new_invalid_error(format!(
        "Viewport {} shows no level to capture",
        id.0
      ))),
      Some(viewport) if viewport.is_capturing() || viewport.is_bursting() => Err(XrfError::new_invalid_error(format!(
        "Viewport {} captures already",
        id.0
      ))),
      Some(viewport) => plan(viewport),
    };

    match (planned, self.viewports.get_mut(&id)) {
      (Ok(plan), Some(viewport)) => viewport.capture = Some(CaptureJob::new(plan, events)),
      (Err(error), _) => {
        let _ = events.send(CaptureEvent::Stopped { error });
      }
      (Ok(_), None) => {}
    }
  }

  /// The scene a viewport asks to show now, as its world says it, ahead of the frame that catches up with it: so a poll
  /// after a switch never reads the scene before as the one asked for.
  fn get_asked_scene(&self, viewport: RenderViewportId) -> Option<RenderSceneId> {
    self
      .world
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .get_asked_scene(viewport)
  }

  /// The scene a viewport shows, none where it shows none.
  fn get_shown_scene(&self, viewport: RenderViewportId) -> Option<&LevelScene> {
    self.scenes.get(&self.viewports.get(&viewport)?.shown.as_ref()?.0)
  }
}
