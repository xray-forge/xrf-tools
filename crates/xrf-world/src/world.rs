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
use crate::contract::world_level_reload::WorldLevelReload;
use crate::contract::world_model_pose::WorldModelPose;
use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::contract::world_toggles::WorldToggles;
use crate::contract::world_weather_control::WorldWeatherControl;
use crate::contract::world_weather_play::WorldWeatherPlay;
use crate::contract::world_weather_transition::WorldWeatherTransition;
use crate::host::world_event_sink::WorldEventSink;
use crate::host::world_level_source::WorldLevelSource;
use crate::level::effects::effects_frame::EffectsFrame;
use crate::level::world_level::WorldLevel;
use crate::weather::viewport_weather::ViewportWeather;
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

  /// Starts a viewport's world, publishing its camera and weather on `sink`: the one door a viewport's world opens by,
  /// so a command or a frame naming a viewport detached since starts nothing.
  pub fn attach(&mut self, viewport: RenderViewportId, sink: Box<dyn WorldEventSink>) {
    let workers: &RenderWorkers = &self.workers;

    self
      .viewports
      .entry(viewport)
      .or_insert_with(|| {
        let now: Instant = Instant::now();

        WorldViewport::new(ViewportWeather::new(now, workers), now)
      })
      .set_sink(sink);
  }

  pub fn detach(&mut self, viewport: RenderViewportId) {
    self.leave_scene(viewport);
    self.viewports.remove(&viewport);
  }

  /// Sets what of a viewport's world plays from its next frame on.
  pub fn set_toggles(&mut self, viewport: RenderViewportId, toggles: WorldToggles) {
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.toggles = toggles;
    }
  }

  pub fn set_camera(&mut self, viewport: RenderViewportId, camera: WorldCamera) {
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.camera.describe(camera);
    }
  }

  pub fn command_camera(&mut self, viewport: RenderViewportId, command: WorldCameraCommand) {
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.camera.command(command);
    }
  }

  pub fn send_input(&mut self, viewport: RenderViewportId, event: &WorldInputEvent) {
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.camera.input(event);
    }
  }

  /// Shows a level in a viewport, or none, and plays its weather there: the scene of the same level another viewport
  /// shows where its source shares one, else a scene of its own streaming in from the next frame. Answers the scene,
  /// which its load reports name.
  pub fn show_level(
    &mut self,
    viewport: RenderViewportId,
    source: Option<Arc<dyn WorldLevelSource>>,
  ) -> Option<RenderSceneId> {
    // A viewport detached since shows nothing, rather than starting a scene nothing would let go.
    if !self.viewports.contains_key(&viewport) {
      return None;
    }

    self.leave_scene(viewport);
    self.get_viewport(viewport).weather.show(source.clone());

    let source: Arc<dyn WorldLevelSource> = source?;
    let key: Option<String> = source.get_scene_key();
    let id: RenderSceneId = match key.as_deref().and_then(|key| self.find_scene(key)) {
      Some(id) => id,
      None => {
        let id: RenderSceneId = self.take_scene_id();

        self.scenes.insert(
          id,
          WorldScene {
            level: WorldLevel::start(source, &self.workers),
            key,
            viewers: Vec::new(),
          },
        );

        id
      }
    };

    if let Some(scene) = self.scenes.get_mut(&id) {
      scene.viewers.push(viewport);
      scene.follow_driver(&self.viewports);
    }

    self.get_viewport(viewport).scene = Some(id);

    Some(id)
  }

  /// Moves the scene sources share by `key` onto `source`, a next source of the same level, keeping its id and its
  /// viewers: what `reload` names is read again from it and replaces what was drawn once read, and everything else
  /// stays as it is drawn. Answers the scene, none where no scene has the key.
  pub fn reload_scene(
    &mut self,
    key: &str,
    source: Arc<dyn WorldLevelSource>,
    reload: WorldLevelReload,
  ) -> Option<RenderSceneId> {
    let id: RenderSceneId = self.find_scene(key)?;
    let scene: &mut WorldScene = self.scenes.get_mut(&id)?;

    scene.level.reload(Arc::clone(&source), reload, &self.workers);

    for viewer in &scene.viewers {
      if let Some(world) = self.viewports.get_mut(viewer) {
        world.weather.follow(Arc::clone(&source), reload.is_weather);
      }
    }

    Some(id)
  }

  /// Streams the scene sources share by `key` afresh from `source`, a level read again whole, as a scene of its own:
  /// every viewport showing the one before shows it, its weather read again, and the one before goes. Answers the new
  /// scene, none where no scene has the key.
  pub fn reopen_scene(&mut self, key: &str, source: Arc<dyn WorldLevelSource>) -> Option<RenderSceneId> {
    let before: RenderSceneId = self.find_scene(key)?;
    let WorldScene { key, viewers, .. } = self.scenes.remove(&before)?;
    let id: RenderSceneId = self.take_scene_id();
    let mut scene: WorldScene = WorldScene {
      level: WorldLevel::start(Arc::clone(&source), &self.workers),
      key,
      viewers,
    };

    scene.follow_driver(&self.viewports);

    for viewer in &scene.viewers {
      if let Some(world) = self.viewports.get_mut(viewer) {
        world.scene = Some(id);
        world.weather.show(Some(Arc::clone(&source)));
      }
    }

    self.scenes.insert(id, scene);

    Some(id)
  }

  /// Stands a viewport's skinned objects as asked from the next frame on; a scene it shares stands them as its driver
  /// asks, as every other choice of a viewport reaches a shared scene.
  pub fn pose_model(&mut self, viewport: RenderViewportId, pose: &WorldModelPose) {
    let Some(world) = self.viewports.get_mut(&viewport) else {
      return;
    };

    world.pose = pose.clone();

    if let Some(scene) = world
      .scene
      .and_then(|id| self.scenes.get_mut(&id))
      .filter(|scene| scene.get_driver() == Some(viewport))
    {
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
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.weather.play(play, transition);
    }
  }

  pub fn set_weather_control(&mut self, viewport: RenderViewportId, control: WorldWeatherControl) {
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.weather.set_control(control);
    }
  }

  pub fn seek_weather(&mut self, viewport: RenderViewportId, time: f32) {
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.weather.seek(time);
    }
  }

  pub fn play_weather_effect(&mut self, viewport: RenderViewportId, name: Option<&str>) {
    if let Some(world) = self.viewports.get_mut(&viewport) {
      world.weather.play_effect(name);
    }
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

  /// The scene sources share by `key`.
  fn find_scene(&self, key: &str) -> Option<RenderSceneId> {
    self
      .scenes
      .iter()
      .find(|(_, scene)| scene.key.as_deref() == Some(key))
      .map(|(id, _)| *id)
  }

  /// The id the next scene takes, which no scene took before.
  fn take_scene_id(&mut self) -> RenderSceneId {
    let id: RenderSceneId = RenderSceneId(self.next_scene);

    self.next_scene += 1;

    id
  }

  /// Takes a viewport out of the scene it shows, which goes once no viewport shows it; the next shows on drive it,
  /// standing its skinned objects as that viewport asks.
  fn leave_scene(&mut self, viewport: RenderViewportId) {
    let Some(id) = self.viewports.get_mut(&viewport).and_then(|world| world.scene.take()) else {
      return;
    };

    if let Some(scene) = self.scenes.get_mut(&id) {
      let was_driving: bool = scene.get_driver() == Some(viewport);

      scene.viewers.retain(|it| *it != viewport);

      if scene.viewers.is_empty() {
        self.scenes.remove(&id);
      } else if was_driving {
        scene.follow_driver(&self.viewports);
      }
    }
  }

  /// An attached viewport's world, which `show_level` has checked it is.
  fn get_viewport(&mut self, viewport: RenderViewportId) -> &mut WorldViewport {
    self.viewports.get_mut(&viewport).expect("the viewport is attached")
  }
}

impl RenderWorld for World {
  fn get_asked_scene(&self, viewport: RenderViewportId) -> Option<RenderSceneId> {
    self.viewports.get(&viewport)?.scene
  }

  fn advance_view(&mut self, viewport: RenderViewportId, input: RenderViewInput<'_>) -> RenderViewFrame {
    let ambient: Option<WorldAmbientReport> = self.get_scene(viewport).and_then(|scene| scene.level.report_ambient());

    // A frame drawn before the world's attach, or after its detach, rests: it starts no world.
    self
      .viewports
      .get_mut(&viewport)
      .map_or_else(RenderViewFrame::default, |world| world.advance(input, ambient))
  }

  fn advance_scenes(
    &mut self,
    now: Instant,
    mut feedback: HashMap<RenderSceneId, RenderSceneFeedback>,
  ) -> HashMap<RenderSceneId, RenderSceneFrame> {
    let mut frames: HashMap<RenderSceneId, RenderSceneFrame> = HashMap::with_capacity(self.scenes.len());

    for (id, scene) in &mut self.scenes {
      let RenderSceneFeedback {
        sector_failures,
        model_failures,
        finished_effects,
      } = feedback.remove(id).unwrap_or_default();

      // The scene told it once, so it is taken even in a frame no viewport drives the scene.
      scene
        .level
        .take_feedback((sector_failures, model_failures), finished_effects);

      let driver: Option<RenderViewportId> = scene.get_driver();
      let Some(driving) = driver.and_then(|driver| self.viewports.get(&driver)) else {
        continue;
      };
      let (updates, effects): (Vec<RenderSceneUpdate>, EffectsFrame) = scene.level.advance(
        now,
        (&driving.options, &driving.toggles, driving.get_ambient_frame()),
        driving.eye,
      );

      frames.insert(
        *id,
        RenderSceneFrame {
          source: Arc::clone(scene.level.get_source()) as Arc<dyn RenderLevelSource>,
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

  fn restart_scenes(&mut self) {
    for scene in self.scenes.values_mut() {
      let source: Arc<dyn WorldLevelSource> = Arc::clone(scene.level.get_source());

      scene.level = WorldLevel::start(source, &self.workers);
      scene.follow_driver(&self.viewports);
    }
  }
}
