use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use xrf_renderer::{
  LoaderReceiver, RenderLevelSource, RenderLoadFailure, RenderModelSkeleton, RenderSceneUpdate, RenderSectorFailure,
  RenderStreamingProgress, RenderWorkers, StaticModelProxy, StaticObjectProxy, StaticSectorProxy, take_answer,
};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};
use xrf_visual::LightsDescription;

use crate::contract::world_level_problems::WorldLevelProblems;
use crate::contract::world_sector_skip::WorldSectorSkip;
use crate::contract::world_surface_geometry::WorldSurfaceGeometry;
use crate::level::level_animation::LevelAnimation;
use crate::level::level_loader::LevelLoader;
use crate::level::spawn_loader::SpawnLoader;
use crate::level::streamed_model::StreamedModel;
use crate::level::surface_tally::SurfaceTally;

/// Sectors posted to the scene at most each frame, so a level's open spreads over frames rather than stalling one.
const SECTORS_PER_FRAME: usize = 4;

/// Spawned models posted at most in one frame.
const MODELS_PER_FRAME: usize = 16;

/// What streams a level into its scene: the workers reading its sectors, spawned models and lights, how much of what
/// they read is posted a frame, the handles its posts name, which spawn groups stand, and what came in and what could
/// not.
pub struct LevelStreaming {
  source: Arc<dyn RenderLevelSource>,
  sectors: LevelLoader,
  spawn: SpawnLoader,
  lights: Option<LoaderReceiver<Result<LightsDescription, String>>>,
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
  /// The spawn groups left out of the scene, a bit a group from the lowest.
  hidden_groups: u32,
  /// The sectors that could not be read or taken in, and the drawables the packer left out of those that were.
  failed_sectors: Vec<RenderLoadFailure>,
  skipped: Vec<WorldSectorSkip>,
  surfaces: SurfaceTally,
  /// When it began opening the level, which the load is timed from.
  started: Instant,
}

impl LevelStreaming {
  pub fn start(source: Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();
    let lights_source: Arc<dyn RenderLevelSource> = Arc::clone(&source);

    workers.spawn(move || {
      let lights: Result<LightsDescription, String> = lights_source.read_lights().map_err(|error| error.to_string());

      if let Err(error) = &lights {
        log::error!("The level's lights cannot be drawn: {error}");
      }

      let _ = sender.send(lights);
    });

    Self {
      sectors: LevelLoader::start(Arc::clone(&source), workers),
      spawn: SpawnLoader::start(Arc::clone(&source), workers),
      lights: Some(receiver),
      light_campfires: Vec::new(),
      light_motions: Vec::new(),
      sector_handles: ProxyAllocator::new(),
      model_handles: ProxyAllocator::new(),
      object_handles: ProxyAllocator::new(),
      posted: HashMap::new(),
      models: Vec::new(),
      hidden_groups: 0,
      failed_sectors: Vec::new(),
      skipped: Vec::new(),
      surfaces: SurfaceTally::default(),
      started: Instant::now(),
      source,
    }
  }

  pub fn get_source(&self) -> &Arc<dyn RenderLevelSource> {
    &self.source
  }

  /// Posts this frame's changes: the spawn groups `hidden` names left out and those it no longer names stood again, the
  /// lights once read, and a few of the sectors and spawned models the workers finished. `failures` are the sectors
  /// the scene could not take in of the last frame's posts.
  pub fn stream(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    animation: &mut LevelAnimation,
    (hidden, failures): (u32, Vec<RenderSectorFailure>),
  ) {
    for failure in failures {
      if let Some((sector, _)) = self.posted.remove(&failure.handle) {
        self.sector_handles.free(failure.handle);
        self.failed_sectors.push(RenderLoadFailure {
          name: sector.to_string(),
          reason: failure.reason,
        });
      }
    }

    self.filter_spawn(updates, animation, hidden);

    if let Some(Ok(lights)) = take_answer(&mut self.lights, "lights") {
      self.light_campfires = lights.lights.iter().filter_map(|light| light.campfire).collect();
      self.light_motions = lights
        .lights
        .iter()
        .filter_map(|light| light.motion.as_ref().map(|motion| motion.name.clone()))
        .collect();
      updates.push(RenderSceneUpdate::AddLights(lights));
    }

    for (sector, package) in self.sectors.take(SECTORS_PER_FRAME) {
      match package {
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

    for (model, places, skeleton) in self.spawn.take(MODELS_PER_FRAME) {
      let handle: ProxyHandle<StaticModelProxy> = self.model_handles.allocate();
      let skeleton: Option<RenderModelSkeleton> = skeleton.filter(|_| model.skin.is_some());

      updates.push(RenderSceneUpdate::AddModel { handle, model });

      let mut streamed: StreamedModel = StreamedModel {
        handle,
        places: places.into_iter().map(|place| (place, None)).collect(),
        skeleton,
      };

      for index in 0..streamed.places.len() {
        if is_group_shown(self.hidden_groups, streamed.places[index].0.group) {
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
    WorldLevelProblems {
      skipped: self.skipped.clone(),
      sectors: self.failed_sectors.clone(),
      models: self.spawn.list_failures(),
    }
  }

  /// Leaves the spawn groups `hidden` names out of the scene: the objects of a group newly hidden leave it, and those of
  /// one newly shown stand in it again.
  fn filter_spawn(&mut self, updates: &mut Vec<RenderSceneUpdate>, animation: &mut LevelAnimation, hidden: u32) {
    let changed: u32 = hidden ^ self.hidden_groups;

    if changed == 0 {
      return;
    }

    self.hidden_groups = hidden;

    let mut models: Vec<StreamedModel> = std::mem::take(&mut self.models);

    for model in &mut models {
      for index in 0..model.places.len() {
        let group: u32 = model.places[index].0.group;

        if group == 0 || changed & (1 << (group - 1)) == 0 {
          continue;
        }

        if is_group_shown(hidden, group) {
          self.stand(updates, animation, model, index);
        } else if let Some(object) = model.places[index].1.take() {
          animation.remove(object);
          self.object_handles.free(object);
          updates.push(RenderSceneUpdate::RemoveObject(object));
        }
      }
    }

    self.models = models;
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

/// Whether a spawn group stands in the scene: group zero always does.
fn is_group_shown(hidden: u32, group: u32) -> bool {
  group == 0 || hidden & (1 << (group - 1)) == 0
}
