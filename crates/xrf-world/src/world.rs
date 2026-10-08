use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use xrf_renderer::{
  RenderLevelSource, RenderSceneFeedback, RenderSceneFrame, RenderSceneId, RenderSceneUpdate, RenderViewFrame,
  RenderViewInput, RenderViewportId, RenderWorkers, RenderWorld,
};

use crate::contract::world_ambient_report::WorldAmbientReport;
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
use crate::level::effects_frame::EffectsFrame;
use crate::level::world_level::WorldLevel;
use crate::weather::viewport_weather::ViewportWeather;
use crate::world_event_sink::WorldEventSink;
use crate::world_scene::WorldScene;
use crate::world_viewport::WorldViewport;

/// What moves and plays in the levels the viewports show: each viewport's camera and its level's weather as it plays
/// there, and each level's scene, streamed, posed and played once for every viewport showing it, driven by the first.
/// The host steers it; the render thread runs it before each frame and draws what it answers.
pub struct World {
  viewports: HashMap<RenderViewportId, WorldViewport>,
  scenes: HashMap<RenderSceneId, WorldScene>,
  /// The id the next scene takes; none is named again.
  next_scene: u32,
  /// The pool its levels and weather are read on.
  workers: RenderWorkers,
}

impl World {
  pub fn new(workers: RenderWorkers) -> Self {
    Self {
      viewports: HashMap::new(),
      scenes: HashMap::new(),
      next_scene: 1,
      workers,
    }
  }

  /// Starts a viewport's world, publishing its camera and weather on `sink`.
  pub fn attach(&mut self, viewport: RenderViewportId, sink: Box<dyn WorldEventSink>) {
    self.get_viewport(viewport).set_sink(sink);
  }

  pub fn detach(&mut self, viewport: RenderViewportId) {
    self.leave_scene(viewport);
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

  /// Shows a level in a viewport, or none, and plays its weather there: the scene of the same level another viewport
  /// shows where its source shares one, else a scene of its own streaming in from the next frame.
  pub fn show_level(&mut self, viewport: RenderViewportId, source: Option<Arc<dyn RenderLevelSource>>) {
    self.leave_scene(viewport);
    self.get_viewport(viewport).weather.show(source.clone());

    let Some(source) = source else {
      return;
    };
    let key: Option<String> = source.get_scene_key();
    let shared: Option<RenderSceneId> = key.as_ref().and_then(|key| {
      self
        .scenes
        .iter()
        .find(|(_, scene)| scene.key.as_ref() == Some(key))
        .map(|(id, _)| *id)
    });
    let id: RenderSceneId = match shared {
      Some(id) => id,
      None => {
        let id: RenderSceneId = RenderSceneId(self.next_scene);
        let mut level: WorldLevel = WorldLevel::start(source, &self.workers);

        self.next_scene += 1;
        level.set_pose(&self.get_viewport(viewport).pose);
        self.scenes.insert(
          id,
          WorldScene {
            level,
            key,
            viewers: Vec::new(),
          },
        );

        id
      }
    };

    if let Some(scene) = self.scenes.get_mut(&id) {
      scene.viewers.push(viewport);
    }

    self.get_viewport(viewport).scene = Some(id);
  }

  /// Stands a viewport's skinned objects as asked from the next frame on.
  pub fn pose_model(&mut self, viewport: RenderViewportId, pose: &WorldModelPose) {
    let world: &mut WorldViewport = self.get_viewport(viewport);

    world.pose = pose.clone();

    if let Some(scene) = world.scene.and_then(|id| self.scenes.get_mut(&id)) {
      scene.level.set_pose(pose);
    }
  }

  /// How much each shader table entry draws across the sectors a viewport's level streamed in.
  pub fn measure_surfaces(&self, viewport: RenderViewportId) -> Vec<WorldSurfaceGeometry> {
    self
      .get_scene(viewport)
      .map(|scene| scene.level.measure_surfaces())
      .unwrap_or_default()
  }

  /// What a viewport's level could not draw the way it asked, so far.
  pub fn describe_problems(&self, viewport: RenderViewportId) -> WorldLevelProblems {
    self
      .get_scene(viewport)
      .map(|scene| scene.level.describe_problems())
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

  /// Plays a weather ambient effect near the camera of the viewport driving a viewport's scene on its next frame,
  /// ending the one playing; none plays indoors.
  pub fn play_ambient_effect(&mut self, viewport: RenderViewportId) {
    let scene: Option<RenderSceneId> = self.viewports.get(&viewport).and_then(|world| world.scene);

    if let Some(scene) = scene.and_then(|id| self.scenes.get_mut(&id)) {
      scene.level.play_ambient_now();
    }
  }

  /// The scene a viewport shows.
  fn get_scene(&self, viewport: RenderViewportId) -> Option<&WorldScene> {
    self.scenes.get(&self.viewports.get(&viewport)?.scene?)
  }

  /// Takes a viewport out of the scene it shows, which goes once no viewport shows it; the next shows on drive it.
  fn leave_scene(&mut self, viewport: RenderViewportId) {
    let Some(id) = self.viewports.get_mut(&viewport).and_then(|world| world.scene.take()) else {
      return;
    };

    if let Some(scene) = self.scenes.get_mut(&id) {
      scene.viewers.retain(|it| *it != viewport);

      if scene.viewers.is_empty() {
        self.scenes.remove(&id);
      }
    }
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
  fn get_asked_scene(&self, viewport: RenderViewportId) -> Option<RenderSceneId> {
    self.viewports.get(&viewport)?.scene
  }

  fn advance_view(&mut self, viewport: RenderViewportId, input: RenderViewInput<'_>) -> RenderViewFrame {
    let ambient: Option<WorldAmbientReport> = self.get_scene(viewport).and_then(|scene| scene.level.report_ambient());

    self.get_viewport(viewport).advance(input, ambient)
  }

  fn advance_scenes(
    &mut self,
    mut feedback: HashMap<RenderSceneId, RenderSceneFeedback>,
  ) -> HashMap<RenderSceneId, RenderSceneFrame> {
    let mut frames: HashMap<RenderSceneId, RenderSceneFrame> = HashMap::with_capacity(self.scenes.len());

    for (id, scene) in &mut self.scenes {
      let driver: Option<RenderViewportId> = scene.get_driver();
      let Some(driving) = driver.and_then(|driver| self.viewports.get(&driver)) else {
        continue;
      };
      let RenderSceneFeedback {
        failures,
        finished_effects,
      } = feedback.remove(id).unwrap_or_default();
      let (updates, effects): (Vec<RenderSceneUpdate>, EffectsFrame) = scene.level.advance(
        (&driving.options, &driving.toggles, driving.get_ambient_frame()),
        driving.eye,
        failures,
        finished_effects,
      );

      frames.insert(
        *id,
        RenderSceneFrame {
          source: Arc::clone(scene.level.get_source()),
          driver,
          updates,
          streaming: scene.level.get_progress(),
          skeleton_segments: scene.level.list_segments(),
          gust: effects.gust,
          campfire_shares: effects.campfire_shares,
          motions: effects.motions,
          camera_hemi: effects.camera_hemi,
        },
      );
    }

    frames
  }
}
