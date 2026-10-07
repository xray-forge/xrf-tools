use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_renderer::{
  RenderCameraPose, RenderEventSink, RenderModelPose, RenderViewportEvent, RenderWorldFrame, RenderWorldInput,
  WeatherTextureKind,
};

use crate::camera::camera_controller::CameraController;
use crate::level::world_level::WorldLevel;
use crate::weather::viewport_weather::ViewportWeather;

/// How often a moving camera's pose is published.
const POSE_INTERVAL: Duration = Duration::from_millis(100);

/// One viewport's world: its camera, the level it shows and that level's weather, and what its page was last told of
/// them.
pub struct WorldViewport {
  pub camera: CameraController,
  pub weather: ViewportWeather,
  pub level: Option<WorldLevel>,
  /// How its skinned models stand, whatever level it shows.
  pub pose: RenderModelPose,
  sink: Option<Box<dyn RenderEventSink>>,
  sent_pose: Option<RenderCameraPose>,
  pose_due: Instant,
}

impl WorldViewport {
  pub fn new(weather: ViewportWeather, now: Instant) -> Self {
    Self {
      camera: CameraController::default(),
      weather,
      level: None,
      pose: RenderModelPose::default(),
      sink: None,
      sent_pose: None,
      pose_due: now,
    }
  }

  /// Gives it the page's channel, which its camera's pose and its weather are published on.
  pub fn set_sink(&mut self, sink: Box<dyn RenderEventSink>) {
    self.sink = Some(sink);
  }

  /// Moves the camera and the weather on a frame, publishes what changed, and answers the frame's input.
  pub fn advance(&mut self, input: RenderWorldInput<'_>) -> RenderWorldFrame {
    let RenderWorldInput {
      now,
      delta,
      height,
      options,
      skies,
      ambient,
      failures,
    } = input;

    self.camera.update(delta, height);

    // The weather is weighed where the camera stands, in engine space; a fade waits for the skies it fades into.
    let position: Vec3 = self.camera.get_pose().position.into();
    let is_thundering: bool = options.world.is_thundering && options.mode.is_lit;

    self
      .weather
      .advance(now, [position.x, position.y, -position.z], is_thundering, |lighting| {
        skies.request_sky(&lighting.sky)
      });

    // The keyframe the clock walks to next has its skies fetched before it is reached.
    if let Some(next) = self.weather.get_player().get_next() {
      let keyframe = &next.descriptor;

      for (reference, kind) in [
        (keyframe.sky_texture.as_str(), WeatherTextureKind::Cube),
        (keyframe.sky_texture_env.as_str(), WeatherTextureKind::Cube),
        (keyframe.clouds_texture.as_str(), WeatherTextureKind::Flat),
      ] {
        if !reference.is_empty() && !keyframe.sky_texture.is_empty() {
          skies.prefetch(reference, kind);
        }
      }
    }

    self.publish_pose(now);

    if let Some(report) = self.weather.take_report(now, ambient) {
      self.send(RenderViewportEvent::Weather { report });
    }

    let updates = match &mut self.level {
      Some(level) => level.advance(options.world.get_hidden_spawn_groups(), failures),
      None => Vec::new(),
    };

    RenderWorldFrame {
      level: self.level.as_ref().map(|level| level.get_source().clone()),
      updates,
      streaming: self.level.as_ref().map(WorldLevel::get_progress).unwrap_or_default(),
      skeleton_segments: self.level.as_ref().map(WorldLevel::list_segments).unwrap_or_default(),
      camera: self.camera.get_frame(),
      lighting: self.weather.get_lighting().clone(),
      weather: self.weather.get_level().cloned(),
      clock_rate: self.weather.get_player().get_clock_rate(),
    }
  }

  /// Publishes the camera's pose when it changed, at most every [`POSE_INTERVAL`]; the last change of a motion is
  /// published at the next interval after it stops.
  fn publish_pose(&mut self, now: Instant) {
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

  fn send(&self, event: RenderViewportEvent) {
    if let Some(sink) = &self.sink {
      sink.send(event);
    }
  }
}
