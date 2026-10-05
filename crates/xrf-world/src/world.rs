use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use xrf_renderer::{
  RenderLevelSource, RenderViewportId, RenderWorkers, RenderWorld, RenderWorldFrame, RenderWorldInput,
};

use crate::contract::world_camera::WorldCamera;
use crate::contract::world_camera_command::WorldCameraCommand;
use crate::contract::world_input_event::WorldInputEvent;
use crate::contract::world_level_problems::WorldLevelProblems;
use crate::contract::world_model_pose::WorldModelPose;
use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::contract::world_toggles::WorldToggles;
use crate::contract::world_weather_control::WorldWeatherControl;
use crate::contract::world_weather_play::WorldWeatherPlay;
use crate::contract::world_weather_transition::WorldWeatherTransition;
use crate::level::world_level::WorldLevel;
use crate::weather::viewport_weather::ViewportWeather;
use crate::world_event_sink::WorldEventSink;
use crate::world_viewport::WorldViewport;

/// What moves and plays in the levels the viewports show, a viewport's own: its camera, the level it shows streaming
/// into its scene, and that level's weather and poses. The host steers it; the render thread runs it before each frame
/// and draws what it answers.
pub struct World {
  viewports: HashMap<RenderViewportId, WorldViewport>,
  /// The pool its weather is read on.
  workers: RenderWorkers,
}

impl World {
  pub fn new(workers: RenderWorkers) -> Self {
    Self {
      viewports: HashMap::new(),
      workers,
    }
  }

  /// Starts a viewport's world, publishing its camera and weather on `sink`.
  pub fn attach(&mut self, viewport: RenderViewportId, sink: Box<dyn WorldEventSink>) {
    self.get_viewport(viewport).set_sink(sink);
  }

  pub fn detach(&mut self, viewport: RenderViewportId) {
    self.viewports.remove(&viewport);
  }

  /// Sets what of a viewport's world plays from its next frame on.
  pub fn set_toggles(&mut self, viewport: RenderViewportId, toggles: WorldToggles) {
    self.get_viewport(viewport).toggles = toggles;
  }

  pub fn set_camera(&mut self, viewport: RenderViewportId, camera: WorldCamera) {
    self.get_viewport(viewport).camera.describe(camera);
  }

  pub fn command_camera(&mut self, viewport: RenderViewportId, command: WorldCameraCommand) {
    self.get_viewport(viewport).camera.command(command);
  }

  pub fn send_input(&mut self, viewport: RenderViewportId, event: &WorldInputEvent) {
    self.get_viewport(viewport).camera.input(event);
  }

  /// Shows a level in a viewport, or none: it streams into a scene of its own from the next frame, and its weather plays.
  pub fn show_level(&mut self, viewport: RenderViewportId, source: Option<Arc<dyn RenderLevelSource>>) {
    let mut level: Option<WorldLevel> = source
      .as_ref()
      .map(|source| WorldLevel::start(Arc::clone(source), &self.workers));
    let world: &mut WorldViewport = self.get_viewport(viewport);

    if let Some(level) = &mut level {
      level.set_pose(&world.pose);
    }

    world.weather.show(source);
    world.level = level;
  }

  /// Stands a viewport's skinned objects as asked from the next frame on.
  pub fn pose_model(&mut self, viewport: RenderViewportId, pose: &WorldModelPose) {
    let world: &mut WorldViewport = self.get_viewport(viewport);

    world.pose = pose.clone();

    if let Some(level) = &mut world.level {
      level.set_pose(pose);
    }
  }

  /// How much each shader table entry draws across the sectors a viewport's level streamed in.
  pub fn measure_surfaces(&self, viewport: RenderViewportId) -> Vec<WorldSurfaceGeometry> {
    self
      .get_level(viewport)
      .map(WorldLevel::measure_surfaces)
      .unwrap_or_default()
  }

  /// What a viewport's level could not draw the way it asked, so far.
  pub fn describe_problems(&self, viewport: RenderViewportId) -> WorldLevelProblems {
    self
      .get_level(viewport)
      .map(WorldLevel::describe_problems)
      .unwrap_or_default()
  }

  pub fn play_weather(
    &mut self,
    viewport: RenderViewportId,
    play: WorldWeatherPlay,
    transition: WorldWeatherTransition,
  ) {
    self.get_viewport(viewport).weather.play(play, transition);
  }

  pub fn set_weather_control(&mut self, viewport: RenderViewportId, control: WorldWeatherControl) {
    self.get_viewport(viewport).weather.set_control(control);
  }

  pub fn seek_weather(&mut self, viewport: RenderViewportId, time: f32) {
    self.get_viewport(viewport).weather.seek(time);
  }

  pub fn play_weather_effect(&mut self, viewport: RenderViewportId, name: Option<&str>) {
    self.get_viewport(viewport).weather.play_effect(name);
  }

  /// Plays a weather ambient effect near a viewport's camera on its next frame, ending the one playing; none plays
  /// indoors.
  pub fn play_ambient_effect(&mut self, viewport: RenderViewportId) {
    if let Some(level) = &mut self.get_viewport(viewport).level {
      level.play_ambient_now();
    }
  }

  fn get_level(&self, viewport: RenderViewportId) -> Option<&WorldLevel> {
    self.viewports.get(&viewport)?.level.as_ref()
  }

  /// A viewport's world, started for whatever names it first: a command may arrive before its attach.
  fn get_viewport(&mut self, viewport: RenderViewportId) -> &mut WorldViewport {
    let workers: &RenderWorkers = &self.workers;

    self.viewports.entry(viewport).or_insert_with(|| {
      let now: Instant = Instant::now();

      WorldViewport::new(ViewportWeather::new(now, workers), now)
    })
  }
}

impl RenderWorld for World {
  fn advance(&mut self, viewport: RenderViewportId, input: RenderWorldInput<'_>) -> RenderWorldFrame {
    self.get_viewport(viewport).advance(input)
  }
}
