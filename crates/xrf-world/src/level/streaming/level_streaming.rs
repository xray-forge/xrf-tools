use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use xrf_renderer::{
  LoaderReceiver, RenderLevelDetails, RenderLevelHelpers, RenderLoadFailure, RenderModelFailure, RenderModelSkeleton,
  RenderSceneUpdate, RenderSectorFailure, RenderStreamingProgress, RenderWorkers, StaticModelPlace, StaticModelProxy,
  StaticObjectProxy, StaticSectorProxy, take_answer,
};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};
use xrf_visual::LightsDescription;

use crate::contract::world_level_problems::WorldLevelProblems;
use crate::contract::world_level_reload::WorldLevelReload;
use crate::contract::world_sector_skip::WorldSectorSkip;
use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::host::world_level_source::WorldLevelSource;
use crate::level::animation::level_animation::LevelAnimation;
use crate::level::loading::level_loader::LevelLoader;
use crate::level::loading::sector_load::SectorLoad;
use crate::level::streaming::hidden_spawn_groups::HiddenSpawnGroups;
use crate::level::streaming::spawn_filter::SpawnFilter;
use crate::level::streaming::spawn_load::SpawnLoad;
use crate::level::streaming::spawn_loader::SpawnLoader;
use crate::level::streaming::streamed_model::StreamedModel;
use crate::level::streaming::surface_tally::SurfaceTally;

/// Sectors posted to the scene at most each frame, so a level's open spreads over frames rather than stalling one.
const SECTORS_PER_FRAME: usize = 4;

/// Spawned models posted at most in one frame.
const MODELS_PER_FRAME: usize = 16;

/// What streams a level into its scene: the workers reading its sectors, spawned models, lights and grass, how much of
/// what they read is posted a frame, the handles its posts name, which spawn groups stand, and what came in and what
/// could not.
pub struct LevelStreaming {
  source: Arc<dyn WorldLevelSource>,
  sectors: LevelLoader,
  spawn: SpawnLoader,
  lights: Option<LoaderReceiver<Result<LightsDescription, String>>>,
  grass: Option<LoaderReceiver<Result<Option<RenderLevelDetails>, String>>>,
  helpers: Option<LoaderReceiver<Result<RenderLevelHelpers, String>>>,
  /// The campfires and the object motions the lights follow, by the campfires' ids and the motions' names.
  light_campfires: Vec<u16>,
  light_motions: Vec<String>,
  sector_handles: ProxyAllocator<StaticSectorProxy>,
  model_handles: ProxyAllocator<StaticModelProxy>,
  object_handles: ProxyAllocator<StaticObjectProxy>,
  /// Each sector posted and still held, by its handle: its index and the bytes of its pack.
  posted: HashMap<ProxyHandle<StaticSectorProxy>, (u32, u64)>,
  /// Each spawned model posted, with every object standing as it, whether its group shows it or not.
  models: Vec<StreamedModel>,
  /// The spawn groups left out of the scene.
  hidden_groups: HiddenSpawnGroups,
  /// The spawned objects left out of it one by one, by index, ascending.
  hidden_objects: Vec<u32>,
  /// The sectors that could not be read or taken in, and the drawables the packer left out of those that were.
  failed_sectors: Vec<RenderLoadFailure>,
  /// The spawned models the scene could not take in, which no object stands as.
  failed_models: Vec<RenderLoadFailure>,
  skipped: Vec<WorldSectorSkip>,
  surfaces: SurfaceTally,
  /// When it began opening the level, which the load is timed from.
  started: Instant,
}

impl LevelStreaming {
  pub fn start(source: Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      sectors: LevelLoader::start(Arc::clone(&source), workers),
      spawn: SpawnLoader::start(Arc::clone(&source), workers),
      lights: Some(Self::read_lights(&source, workers)),
      grass: Some(Self::read_grass(&source, workers)),
      helpers: Some(Self::read_helpers(&source, workers)),
      light_campfires: Vec::new(),
      light_motions: Vec::new(),
      sector_handles: ProxyAllocator::new(),
      model_handles: ProxyAllocator::new(),
      object_handles: ProxyAllocator::new(),
      posted: HashMap::new(),
      models: Vec::new(),
      hidden_groups: HiddenSpawnGroups::default(),
      hidden_objects: Vec::new(),
      failed_sectors: Vec::new(),
      failed_models: Vec::new(),
      skipped: Vec::new(),
      surfaces: SurfaceTally::default(),
      started: Instant::now(),
      source,
    }
  }

  /// Reads the level's lights on the workers.
  fn read_lights(
    source: &Arc<dyn WorldLevelSource>,
    workers: &RenderWorkers,
  ) -> LoaderReceiver<Result<LightsDescription, String>> {
    let (sender, receiver) = LoaderReceiver::channel();
    let source: Arc<dyn WorldLevelSource> = Arc::clone(source);

    workers.spawn(move || {
      let lights: Result<LightsDescription, String> = source.read_lights().map_err(|error| error.to_string());

      if let Err(error) = &lights {
        log::error!("The level's lights cannot be drawn: {error}");
      }

      let _ = sender.send(lights);
    });

    receiver
  }

  /// Reads the level's grass on the workers.
  fn read_grass(
    source: &Arc<dyn WorldLevelSource>,
    workers: &RenderWorkers,
  ) -> LoaderReceiver<Result<Option<RenderLevelDetails>, String>> {
    let (sender, receiver) = LoaderReceiver::channel();
    let source: Arc<dyn WorldLevelSource> = Arc::clone(source);

    workers.spawn(move || {
      let grass: Result<Option<RenderLevelDetails>, String> = source.read_details().map_err(|error| error.to_string());

      if let Err(error) = &grass {
        log::warn!("The level's grass cannot be drawn: {error}");
      }

      let _ = sender.send(grass);
    });

    receiver
  }

  /// Reads the level objects the viewer draws helpers for on the workers.
  fn read_helpers(
    source: &Arc<dyn WorldLevelSource>,
    workers: &RenderWorkers,
  ) -> LoaderReceiver<Result<RenderLevelHelpers, String>> {
    let (sender, receiver) = LoaderReceiver::channel();
    let source: Arc<dyn WorldLevelSource> = Arc::clone(source);

    workers.spawn(move || {
      let helpers: Result<RenderLevelHelpers, String> = source.read_helpers().map_err(|error| error.to_string());

      if let Err(error) = &helpers {
        log::warn!("The level's helpers cannot be drawn: {error}");
      }

      let _ = sender.send(helpers);
    });

    receiver
  }

  pub fn get_source(&self) -> &Arc<dyn WorldLevelSource> {
    &self.source
  }

  /// Streams on from `source`, a next source of the same level: the sectors posted stay, and those still being read
  /// finish from the source they started from. Where `reload` names the spawn, every spawned model posted leaves the
  /// scene with its objects and the spawn streams in again; where it names the lights or the grass, they are read again
  /// and replace those drawn once read.
  pub fn follow(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    animation: &mut LevelAnimation,
    source: Arc<dyn WorldLevelSource>,
    reload: WorldLevelReload,
    workers: &RenderWorkers,
  ) {
    self.source = source;

    if reload.is_spawn {
      for model in std::mem::take(&mut self.models) {
        self.release_model(animation, &model);
        updates.push(RenderSceneUpdate::RemoveModel(model.handle));
      }

      self.failed_models.clear();
      self.spawn = SpawnLoader::start(Arc::clone(&self.source), workers);
      self.helpers = Some(Self::read_helpers(&self.source, workers));
    }

    if reload.is_lights {
      self.lights = Some(Self::read_lights(&self.source, workers));
    }

    if reload.is_grass {
      self.grass = Some(Self::read_grass(&self.source, workers));
    }
  }

  /// Records the sectors and the spawned models the scene could not take in of the posts before, forgetting them.
  pub fn take_failures(
    &mut self,
    animation: &mut LevelAnimation,
    (failures, model_failures): (Vec<RenderSectorFailure>, Vec<RenderModelFailure>),
  ) {
    self.forget_failed_models(animation, model_failures);

    for failure in failures {
      if let Some((sector, _)) = self.posted.remove(&failure.handle) {
        self.sector_handles.free(failure.handle);
        self.failed_sectors.push(RenderLoadFailure {
          name: sector.to_string(),
          reason: failure.reason,
        });
      }
    }
  }

  /// Posts this frame's changes: the spawned objects `hidden` leaves out taken out and those it no longer does stood
  /// again, the lights and the grass once read, and a few of the sectors and spawned models the workers finished.
  pub fn stream(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    animation: &mut LevelAnimation,
    hidden: SpawnFilter<'_>,
  ) {
    self.filter_spawn(updates, animation, hidden);

    if let Some(Ok(grass)) = take_answer(&mut self.grass, "grass") {
      updates.push(RenderSceneUpdate::AddGrass(grass));
    }

    if let Some(Ok(helpers)) = take_answer(&mut self.helpers, "helpers") {
      updates.push(RenderSceneUpdate::SetHelpers(helpers));
    }

    if let Some(Ok(lights)) = take_answer(&mut self.lights, "lights") {
      self.light_campfires = lights.lights.iter().filter_map(|light| light.campfire).collect();
      self.light_motions = lights
        .lights
        .iter()
        .filter_map(|light| light.motion.as_ref().map(|motion| motion.name.clone()))
        .collect();
      updates.push(RenderSceneUpdate::AddLights(lights));
    }

    for SectorLoad { sector, pack } in self.sectors.take(SECTORS_PER_FRAME) {
      match pack {
        Ok((package, tally)) => {
          let handle: ProxyHandle<StaticSectorProxy> = self.sector_handles.allocate();

          self
            .skipped
            .extend(package.description.skipped.iter().map(|skip| WorldSectorSkip {
              sector,
              skip: skip.clone(),
            }));
          self.surfaces.merge(tally);
          self.posted.insert(handle, (sector, package.buffer.len() as u64));
          updates.push(RenderSceneUpdate::AddSector { handle, package });
        }
        Err(reason) => self.failed_sectors.push(RenderLoadFailure {
          name: sector.to_string(),
          reason,
        }),
      }
    }

    for SpawnLoad {
      model,
      name,
      places,
      skeleton,
    } in self.spawn.take(MODELS_PER_FRAME)
    {
      let handle: ProxyHandle<StaticModelProxy> = self.model_handles.allocate();
      let skeleton: Option<RenderModelSkeleton> = skeleton.filter(|_| model.skin.is_some());

      updates.push(RenderSceneUpdate::AddModel { handle, model });

      let mut streamed: StreamedModel = StreamedModel {
        handle,
        name,
        places: places.into_iter().map(|place| (place, None)).collect(),
        skeleton,
      };

      for index in 0..streamed.places.len() {
        let StaticModelPlace { group, object, .. } = streamed.places[index].0;

        if self.get_filter().is_shown(group, object) {
          self.stand(updates, animation, &mut streamed, index);
        }
      }

      self.models.push(streamed);
    }
  }

  /// How far the level has streamed into its scene.
  pub fn get_progress(&self) -> RenderStreamingProgress {
    let sectors: u32 = self.posted.len() as u32;
    let sectors_total: u32 = self.sectors.get_total();

    RenderStreamingProgress {
      started: Some(self.started),
      sectors,
      sectors_total,
      is_sectors_done: sectors + self.failed_sectors.len() as u32 == sectors_total,
      is_spawn_done: self.spawn.is_done(),
      is_lights_done: self.lights.is_none(),
      is_grass_done: self.grass.is_none(),
      bytes: self.posted.values().map(|(_, bytes)| bytes).sum(),
      ..RenderStreamingProgress::default()
    }
  }

  /// The campfires and the object motions the level's lights follow, by the campfires' ids and the motions' names.
  pub fn list_light_followed(&self) -> (&[u16], &[String]) {
    (&self.light_campfires, &self.light_motions)
  }

  /// How much each shader table entry draws across the sectors streamed in.
  pub fn measure_surfaces(&self) -> Vec<WorldSurfaceGeometry> {
    self.surfaces.list()
  }

  /// What the level could not draw the way it asked, so far.
  pub fn describe_problems(&self) -> WorldLevelProblems {
    let mut models: Vec<RenderLoadFailure> = self.spawn.list_failures();

    models.extend_from_slice(&self.failed_models);

    WorldLevelProblems {
      skipped: self.skipped.clone(),
      sectors: self.failed_sectors.clone(),
      models,
    }
  }

  /// Forgets each spawned model the scene could not take in, with every object standing as it, which the scene never
  /// took in either, and records it failed: none of its objects is stood again.
  fn forget_failed_models(&mut self, animation: &mut LevelAnimation, failures: Vec<RenderModelFailure>) {
    for failure in failures {
      let Some(index) = self.models.iter().position(|model| model.handle == failure.handle) else {
        continue;
      };
      let model: StreamedModel = self.models.remove(index);

      self.release_model(animation, &model);
      self.failed_models.push(RenderLoadFailure {
        name: model.name,
        reason: failure.reason,
      });
    }
  }

  /// What the scene leaves out of the spawn now.
  fn get_filter(&self) -> SpawnFilter<'_> {
    SpawnFilter {
      groups: self.hidden_groups,
      objects: &self.hidden_objects,
    }
  }

  /// Leaves what `hidden` names out of the scene: the objects newly hidden, by group or one by one, leave it, and those
  /// newly shown stand in it again. Compared with what was hidden before a frame, a slice compared and copied only once
  /// it changed.
  fn filter_spawn(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    animation: &mut LevelAnimation,
    hidden: SpawnFilter<'_>,
  ) {
    if self.hidden_groups == hidden.groups && self.hidden_objects == hidden.objects {
      return;
    }

    let before_groups: HiddenSpawnGroups = std::mem::replace(&mut self.hidden_groups, hidden.groups);
    let before_objects: Vec<u32> = std::mem::replace(&mut self.hidden_objects, hidden.objects.to_vec());
    let before: SpawnFilter<'_> = SpawnFilter {
      groups: before_groups,
      objects: &before_objects,
    };
    let mut models: Vec<StreamedModel> = std::mem::take(&mut self.models);

    for model in &mut models {
      for index in 0..model.places.len() {
        let StaticModelPlace { group, object, .. } = model.places[index].0;
        let is_shown: bool = hidden.is_shown(group, object);

        if is_shown == before.is_shown(group, object) {
          continue;
        }

        if is_shown {
          self.stand(updates, animation, model, index);
        } else if let Some(object) = model.places[index].1.take() {
          self.release_object(animation, object);
          updates.push(RenderSceneUpdate::RemoveObject(object));
        }
      }
    }

    self.models = models;
  }

  /// Lets go of a model's handle and of every object standing as it, which leave the scene with it.
  fn release_model(&mut self, animation: &mut LevelAnimation, model: &StreamedModel) {
    for object in model.places.iter().filter_map(|(_, object)| *object) {
      self.release_object(animation, object);
    }

    self.model_handles.free(model.handle);
  }

  /// Lets go of an object's handle and stops posing it.
  fn release_object(&mut self, animation: &mut LevelAnimation, object: ProxyHandle<StaticObjectProxy>) {
    animation.remove(object);
    self.object_handles.free(object);
  }

  /// Stands one of a model's objects in the scene, posed by its skeleton where it has one.
  fn stand(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    animation: &mut LevelAnimation,
    model: &mut StreamedModel,
    index: usize,
  ) {
    let (place, standing) = &mut model.places[index];

    if standing.is_some() {
      return;
    }

    let object: ProxyHandle<StaticObjectProxy> = self.object_handles.allocate();

    *standing = Some(object);
    updates.push(RenderSceneUpdate::AddObject {
      handle: object,
      model: model.handle,
      place: *place,
    });

    if let Some(skeleton) = &model.skeleton {
      animation.add(object, skeleton, place.transform);
    }
  }
}
