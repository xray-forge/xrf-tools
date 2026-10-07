use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_renderer_core::ProxyHandle;
use xrf_visual::LightsDescription;

use crate::contract::render_level_problems::RenderLevelProblems;
use crate::contract::render_load_durations::RenderLoadDurations;
use crate::contract::render_load_failure::RenderLoadFailure;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_sector_skip::RenderSectorSkip;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::scene::level::level_loader::LevelLoader;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::loader_answer::take_answer;
use crate::scene::level::posed_skeleton::PosedSkeleton;
use crate::scene::level::spawn_loader::SpawnLoader;
use crate::scene::level::surface_tally::SurfaceTally;
use crate::scene::static_scene::static_model_place::StaticModelPlace;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::texture::texture_cache::TextureCache;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

/// Sectors put on the GPU at most each frame, so a level's open spreads over frames rather than stalling one.
const SECTORS_PER_FRAME: usize = 4;

/// Spawned models put into the scene at most in one frame.
const MODELS_PER_FRAME: usize = 16;

/// What streams a level into its scene: the workers reading its sectors, spawned models and lights, how much of what
/// they read goes in a frame, which spawn groups stand in it, and what the load reports: what came in, what could not,
/// and how long each part took. The scene it fills knows nothing of files, failures or readiness.
pub struct LevelStreaming {
  source: Arc<dyn RenderLevelSource>,
  sectors: LevelLoader,
  spawn: SpawnLoader,
  lights: Option<LoaderReceiver<Result<LightsDescription, String>>>,
  /// Each spawned model in the scene, with every object standing as it, whether its group shows it or not.
  models: Vec<(ProxyHandle<StaticModelProxy>, Vec<StaticModelPlace>)>,
  /// The spawn groups left out of the scene, a bit a group from the lowest.
  hidden_groups: u32,
  /// The sectors that could not be read or put in, and the drawables the packer left out of those that were.
  failed_sectors: Vec<RenderLoadFailure>,
  skipped: Vec<RenderSectorSkip>,
  surfaces: SurfaceTally,
  /// Milliseconds the last sector taken in took to put into the scene.
  sector_time: f32,
  /// When the level began opening, which its load is timed from.
  started: Instant,
  /// How long the level had been opening when each part of it finished.
  load_durations: RenderLoadDurations,
  reported: Option<RenderLoadReport>,
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
      models: Vec::new(),
      hidden_groups: 0,
      failed_sectors: Vec::new(),
      skipped: Vec::new(),
      surfaces: SurfaceTally::default(),
      sector_time: 0.0,
      started: Instant::now(),
      load_durations: RenderLoadDurations::default(),
      reported: None,
      source,
    }
  }

  /// Applies this frame's changes to the scene in one step: the spawn groups `hidden` names left out of it and those it
  /// no longer names put back, the lights once read, and a few of the sectors and spawned models the workers finished.
  #[allow(clippy::too_many_arguments)]
  pub fn stream(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    scene: &mut LevelScene,
    (textures, assets): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
    hidden: u32,
  ) {
    self.filter_spawn((device, queue), scene, hidden);

    if let Some(Ok(lights)) = take_answer(&mut self.lights, "lights") {
      scene.lights.add_lights(lights, textures, assets);
    }

    for (sector, package) in self.sectors.take(SECTORS_PER_FRAME) {
      let (package, tally) = match package {
        Ok(read) => read,
        Err(reason) => {
          self.failed_sectors.push(RenderLoadFailure {
            name: sector.to_string(),
            reason,
          });

          continue;
        }
      };

      self
        .skipped
        .extend(package.description.skipped.iter().map(|skip| RenderSectorSkip {
          sector,
          skip: skip.clone(),
        }));

      let started: Instant = Instant::now();
      let added = scene.statics.add_sector(
        device,
        queue,
        encoder,
        textures,
        assets,
        self.source.get_surfaces(),
        &package,
      );

      self.sector_time = started.elapsed().as_secs_f32() * 1000.0;

      match added {
        Ok(_) => self.surfaces.merge(tally),
        Err(error) => self.failed_sectors.push(RenderLoadFailure {
          name: sector.to_string(),
          reason: error.to_string(),
        }),
      }
    }

    for (model, places, skeleton) in self.spawn.take(MODELS_PER_FRAME) {
      let handle: ProxyHandle<StaticModelProxy> =
        match scene
          .statics
          .add_model(device, queue, encoder, (&mut *textures, assets), &model)
        {
          Ok(handle) => handle,
          Err(error) => {
            log::warn!("Native viewport could not add a spawned model: {error}");

            continue;
          }
        };

      if let Some(skeleton) = skeleton.filter(|_| model.skin.is_some()) {
        for place in &places {
          scene.skeletons.insert(place.object, PosedSkeleton::new(&skeleton));
        }
      }

      for place in places
        .iter()
        .filter(|place| is_group_shown(self.hidden_groups, place.group))
      {
        add_object((device, queue), scene, handle, place);
      }

      self.models.push((handle, places));
    }

    self.note_load_durations(scene, textures);
  }

  /// Whether every sector has been taken in or failed.
  pub fn are_sectors_done(&self, scene: &LevelScene) -> bool {
    (scene.statics.get_sector_count() + self.failed_sectors.len()) as u32 == self.sectors.get_total()
  }

  /// How much each shader table entry draws across the sectors resident.
  pub fn measure_surfaces(&self) -> Vec<RenderSurfaceGeometry> {
    self.surfaces.list()
  }

  /// What the level could not draw the way it asked, so far.
  pub fn describe_problems(&self) -> RenderLevelProblems {
    RenderLevelProblems {
      skipped: self.skipped.clone(),
      sectors: self.failed_sectors.clone(),
      models: self.spawn.list_failures(),
    }
  }

  /// Milliseconds the last sector taken in took to put into the scene.
  pub fn get_sector_time(&self) -> f32 {
    self.sector_time
  }

  /// Whether everything the level opens with is resident, so it draws as it will.
  pub fn is_ready(&self, scene: &LevelScene, textures: &TextureCache) -> bool {
    self.describe_load(scene, textures).is_ready
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_report(&mut self, scene: &LevelScene, textures: &TextureCache) -> Option<RenderLoadReport> {
    let report: RenderLoadReport = self.describe_load(scene, textures);

    if self.reported == Some(report) {
      return None;
    }

    self.reported = Some(report);

    Some(report)
  }

  /// How far the level has loaded: its sectors taken in or failed, its spawn, its grass, lights and particles read, and
  /// every texture it samples settled; and how long each took.
  pub fn describe_load(&self, scene: &LevelScene, textures: &TextureCache) -> RenderLoadReport {
    let settled: u32 = textures.count_settled(scene.list_texture_slots());
    let total: u32 = scene.list_texture_slots().count() as u32;
    let is_read: bool =
      self.spawn.is_done() && scene.grass.is_loaded() && self.lights.is_none() && scene.particles.is_loaded();

    RenderLoadReport {
      sectors: scene.statics.get_sector_count() as u32,
      sectors_total: self.sectors.get_total(),
      bytes: scene.statics.get_bytes(),
      textures: settled,
      textures_total: total,
      is_ready: self.are_sectors_done(scene) && is_read && settled == total,
      durations: self.load_durations,
    }
  }

  /// Notes how long the level had been opening when each part of it finished, the first frame it is seen finished.
  fn note_load_durations(&mut self, scene: &LevelScene, textures: &TextureCache) {
    // Every part has finished by the time the whole has.
    if self.load_durations.ready.is_some() {
      return;
    }

    let elapsed: Duration = self.started.elapsed();
    let finished: [bool; 6] = [
      self.are_sectors_done(scene),
      self.spawn.is_done(),
      scene.grass.is_loaded(),
      self.lights.is_none(),
      scene.particles.is_loaded(),
      self.describe_load(scene, textures).is_ready,
    ];
    let RenderLoadDurations {
      sectors,
      spawn,
      grass,
      lights,
      particles,
      ready,
    } = &mut self.load_durations;

    for (duration, is_finished) in [sectors, spawn, grass, lights, particles, ready]
      .into_iter()
      .zip(finished)
    {
      if is_finished {
        duration.get_or_insert(elapsed);
      }
    }
  }

  /// Leaves the spawn groups `hidden` names out of the scene: the objects of a group newly hidden leave it, and those of
  /// one newly shown stand in it again.
  fn filter_spawn(&mut self, gpu: (&wgpu::Device, &wgpu::Queue), scene: &mut LevelScene, hidden: u32) {
    let changed: u32 = hidden ^ self.hidden_groups;

    if changed == 0 {
      return;
    }

    self.hidden_groups = hidden;

    for (handle, places) in &self.models {
      for place in places
        .iter()
        .filter(|place| place.group != 0 && changed & (1 << (place.group - 1)) != 0)
      {
        if is_group_shown(hidden, place.group) {
          add_object(gpu, scene, *handle, place);
        } else {
          scene.statics.remove_object_by_index(place.object);
        }
      }
    }
  }
}

/// Stands a spawned object in the scene, saying so where it cannot.
fn add_object(
  (device, queue): (&wgpu::Device, &wgpu::Queue),
  scene: &mut LevelScene,
  model: ProxyHandle<StaticModelProxy>,
  place: &StaticModelPlace,
) {
  if let Err(error) = scene.statics.add_object(device, queue, model, place) {
    log::warn!(
      "Native viewport could not stand spawned object {}: {error}",
      place.object
    );
  }
}

/// Whether a spawn group stands in the scene: group zero always does.
fn is_group_shown(hidden: u32, group: u32) -> bool {
  group == 0 || hidden & (1 << (group - 1)) == 0
}
