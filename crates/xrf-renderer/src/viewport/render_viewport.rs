use std::time::{Duration, Instant};

use crate::camera::camera_controller::CameraController;
use crate::contract::render_camera_pose::RenderCameraPose;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_viewport_event::RenderViewportEvent;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::contract::render_viewport_layout::RenderViewportLayout;
use crate::frame::frame_statistics::{FrameStatistics, FrameSummary};
use crate::host::render_event_sink::RenderEventSink;
use crate::pass::view_binding::ViewBinding;

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
  /// Its camera on the GPU, made once a GPU is there.
  pub binding: Option<ViewBinding>,
  sink: Box<dyn RenderEventSink>,
  statistics: FrameStatistics,
  sent_pose: Option<RenderCameraPose>,
  pose_due: Instant,
  /// Whether its page stopped listening, after which it is detached.
  is_gone: bool,
  /// Whether it was told the renderer cannot draw, so it is told once.
  is_failed: bool,
}

impl RenderViewport {
  pub fn new(id: RenderViewportId, window: u64, sink: Box<dyn RenderEventSink>, now: Instant) -> Self {
    Self {
      id,
      window,
      layout: None,
      camera: CameraController::default(),
      binding: None,
      sink,
      statistics: FrameStatistics::new(now),
      sent_pose: None,
      pose_due: now,
      is_gone: false,
      is_failed: false,
    }
  }

  /// Where it is drawn in a window frame of the given size, or `None` when nothing of it shows.
  pub fn get_drawn_rect(&self, width: u32, height: u32) -> Option<RenderRect> {
    self.layout.and_then(|layout| layout.rect.clip(width, height))
  }

  /// The viewport's height in CSS pixels, which drags are measured in.
  pub fn get_css_height(&self) -> f32 {
    self
      .layout
      .map_or(1.0, |layout| layout.rect.height as f32 / layout.scale.max(0.1))
  }

  pub fn is_gone(&self) -> bool {
    self.is_gone
  }

  pub fn record_frame(&mut self, interval: Duration, cpu: Duration) {
    self.statistics.record(interval, cpu);
  }

  /// Reports the frames since the last report, once one is due.
  pub fn report(&mut self, now: Instant, backend: &str, adapter: &str) {
    let Some(summary) = self.statistics.take(now) else {
      return;
    };
    let FrameSummary {
      frames_per_second,
      frame_time,
      frame_time_max,
      cpu_time,
    } = summary;
    let rect: RenderRect = self.layout.map(|layout| layout.rect).unwrap_or_default();

    self.send(RenderViewportEvent::Frame {
      report: RenderFrameReport {
        frames_per_second,
        frame_time,
        frame_time_max,
        cpu_time,
        width: rect.width,
        height: rect.height,
        backend: backend.to_string(),
        adapter: adapter.to_string(),
      },
    });
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
