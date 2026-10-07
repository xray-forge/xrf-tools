use std::time::{Duration, Instant};

use glam::Vec3;
use xrf_renderer::{RenderLighting, RenderSceneUpdate, RenderWorldFrame, RenderWorldInput, WeatherTextureKind};

use crate::camera::camera_controller::CameraController;
use crate::contract::world_camera_pose::WorldCameraPose;
use crate::contract::world_model_pose::WorldModelPose;
use crate::contract::world_toggles::WorldToggles;
use crate::contract::world_viewport_event::WorldViewportEvent;
use crate::level::ambient_frame::AmbientFrame;
use crate::level::effects_frame::EffectsFrame;
use crate::level::world_level::WorldLevel;
use crate::weather::viewport_weather::ViewportWeather;
use crate::world_event_sink::WorldEventSink;

/// How often a moving camera's pose is published.
const POSE_INTERVAL: Duration = Duration::from_millis(100);

/// One viewport's world: its camera, the level it shows and that level's weather, and what its page was last told of
/// them.
pub struct WorldViewport {
  pub camera: CameraController,
  pub weather: ViewportWeather,
  pub level: Option<WorldLevel>,
  /// How its skinned models stand, whatever level it shows.
  pub pose: WorldModelPose,
  /// What of its world plays: the weather's rain, bolts and wind, the campfires, the ambient effects, and which spawn
  /// groups stream in.
  pub toggles: WorldToggles,
  sink: Option<Box<dyn WorldEventSink>>,
  sent_pose: Option<WorldCameraPose>,
  pose_due: Instant,
}

impl WorldViewport {
  pub fn new(weather: ViewportWeather, now: Instant) -> Self {
    Self {
      camera: CameraController::default(),
      weather,
      level: None,
      pose: WorldModelPose::default(),
      toggles: WorldToggles::default(),
      sink: None,
      sent_pose: None,
      pose_due: now,
    }
  }

  /// Gives it the page's channel, which its camera's pose and its weather are published on.
  pub fn set_sink(&mut self, sink: Box<dyn WorldEventSink>) {
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
      failures,
      finished_effects,
    } = input;

    self.camera.update(delta, height);

    // The weather is weighed where the camera stands, in engine space; a fade waits for the skies it fades into.
    let position: Vec3 = self.camera.get_pose().position.into();
    let is_thundering: bool = self.toggles.is_thundering && options.mode.is_lit;

    let eye: Vec3 = Vec3::new(position.x, position.y, -position.z);

    self.weather.advance(now, eye.to_array(), is_thundering, |lighting| {
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

    let ambient: Option<AmbientFrame<'_>> = self.weather.get_level().map(|level| AmbientFrame {
      ambients: &self.weather.get_lighting().ambients,
      level,
    });
    let (updates, effects): (Vec<RenderSceneUpdate>, EffectsFrame) = match &mut self.level {
      Some(level) => level.advance((options, &self.toggles, ambient), eye, failures, finished_effects),
      None => (Vec::new(), EffectsFrame::default()),
    };

    if let Some(report) = self
      .weather
      .take_report(now, self.level.as_ref().and_then(WorldLevel::report_ambient))
    {
      self.send(WorldViewportEvent::Weather { report });
    }

    RenderWorldFrame {
      level: self.level.as_ref().map(|level| level.get_source().clone()),
      updates,
      streaming: self.level.as_ref().map(WorldLevel::get_progress).unwrap_or_default(),
      skeleton_segments: self.level.as_ref().map(WorldLevel::list_segments).unwrap_or_default(),
      camera: self.camera.get_frame(),
      lighting: self.get_lighting(),
      weather: self.weather.get_level().cloned(),
      clock_rate: self.weather.get_player().get_clock_rate(),
      gust: effects.gust,
      campfire_shares: effects.campfire_shares,
      motions: effects.motions,
    }
  }

  /// One frame's lighting as the toggles let it play: no rain while it is switched off, and trees and grass standing
  /// still without the wind.
  fn get_lighting(&self) -> RenderLighting {
    let mut lighting: RenderLighting = self.weather.get_lighting().clone();

    if !self.toggles.is_rainy {
      lighting.rain = None;
    }

    if !self.toggles.is_windy {
      lighting.trees = None;
    }

    lighting
  }

  /// Publishes the camera's pose when it changed, at most every [`POSE_INTERVAL`]; the last change of a motion is
  /// published at the next interval after it stops.
  fn publish_pose(&mut self, now: Instant) {
    if now < self.pose_due {
      return;
    }

    let pose: WorldCameraPose = self.camera.get_pose();

    if self.sent_pose != Some(pose) {
      self.sent_pose = Some(pose);
      self.pose_due = now + POSE_INTERVAL;
      self.send(WorldViewportEvent::Camera { pose });
    }
  }

  fn send(&self, event: WorldViewportEvent) {
    if let Some(sink) = &self.sink {
      sink.send(event);
    }
  }
}
