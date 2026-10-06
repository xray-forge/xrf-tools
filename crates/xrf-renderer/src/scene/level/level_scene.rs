use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::contract::render_load_durations::RenderLoadDurations;
use crate::contract::render_load_failure::RenderLoadFailure;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_model_pose::RenderModelPose;
use crate::contract::render_sector_skip::RenderSectorSkip;
use crate::host::render_level_source::RenderLevelSource;
use crate::scene::level::level_campfires::LevelCampfires;
use crate::scene::level::level_grass::LevelGrass;
use crate::scene::level::level_lights::LevelLights;
use crate::scene::level::level_loader::LevelLoader;
use crate::scene::level::level_object_motions::LevelObjectMotions;
use crate::scene::level::level_particles::LevelParticles;
use crate::scene::level::model_motions::ModelMotions;
use crate::scene::level::posed_skeleton::PosedSkeleton;
use crate::scene::level::spawn_loader::SpawnLoader;
use crate::scene::level::surface_tally::SurfaceTally;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::thread::render_workers::RenderWorkers;

/// A level as the renderer holds it, whatever views draw it: its static geometry, grass, lights and the effects and
/// motions living in it, the models its weather draws, and, until a streaming layer takes them, its loaders and what
/// its load reports.
pub struct LevelScene {
  pub source: Arc<dyn RenderLevelSource>,
  pub loader: LevelLoader,
  pub spawn: SpawnLoader,
  pub grass: LevelGrass,
  pub statics: StaticScene,
  /// The cubes the scene's environment-mapped models mix toward, by environment slot from the second, as of the cache's
  /// generation of them.
  pub environments: (u64, Vec<String>),
  /// The splash's model, with the level's weather it was built for.
  pub splash: Option<(usize, WeatherModelBuffers)>,
  /// Every bolt model of the level's weather, with the weather they were built for, and an empty one the glows bind.
  pub thunder_models: Option<(usize, Vec<WeatherModelBuffers>)>,
  pub no_model: WeatherModelBuffers,
  /// When the level began opening, which the clouds drift from and its load is timed from.
  pub started: Instant,
  pub lights: LevelLights,
  pub campfires: LevelCampfires,
  /// The object motions its moving zones follow, which their particles and lights both read.
  pub object_motions: LevelObjectMotions,
  pub particles: LevelParticles,
  pub surfaces: SurfaceTally,
  /// Each skinned object's skeleton, by its index, the motions they are posed by, and the pose asked for.
  pub skeletons: HashMap<u32, PosedSkeleton>,
  pub motions: ModelMotions,
  pub model_pose: RenderModelPose,
  /// The sectors that could not be read, and the drawables the packer left out of those that were.
  pub failed_sectors: Vec<RenderLoadFailure>,
  pub skipped: Vec<RenderSectorSkip>,
  /// Milliseconds the last sector taken in took to put into the scene.
  pub sector_time: f32,
  /// How long the level had been opening when each part of it finished.
  pub load_durations: RenderLoadDurations,
  pub reported: Option<RenderLoadReport>,
}

impl LevelScene {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    source: Arc<dyn RenderLevelSource>,
    workers: &RenderWorkers,
  ) -> Self {
    // Taken before anything is made, so the load is timed from the moment the level began opening.
    let started: Instant = Instant::now();
    let statics: StaticScene = StaticScene::new(device, queue);

    Self {
      loader: LevelLoader::start(Arc::clone(&source), workers),
      spawn: SpawnLoader::start(Arc::clone(&source), workers),
      grass: LevelGrass::new(device, &source, workers),
      lights: LevelLights::new(device, view_layout, statics.args.size(), &source, workers),
      campfires: LevelCampfires::new(),
      object_motions: LevelObjectMotions::new(&source, workers),
      particles: LevelParticles::new(device, &source, workers),
      statics,
      environments: (0, Vec::new()),
      splash: None,
      thunder_models: None,
      no_model: WeatherModelBuffers::new(device, None),
      started,
      surfaces: SurfaceTally::default(),
      skeletons: HashMap::new(),
      motions: ModelMotions::new(workers),
      model_pose: RenderModelPose::default(),
      failed_sectors: Vec::new(),
      skipped: Vec::new(),
      sector_time: 0.0,
      load_durations: RenderLoadDurations::default(),
      reported: None,
      source,
    }
  }
}
