use std::collections::{BTreeSet, HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::{Vec3, Vec4};
use xrf_error::XrfResult;
use xrf_renderer_core::ProxyHandle;
use xrf_visual::SectorPackage;

use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_texture_group::RenderTextureGroup;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_model_failure::RenderModelFailure;
use crate::host::render_scene_feedback::RenderSceneFeedback;
use crate::host::render_scene_frame::RenderSceneFrame;
use crate::host::render_scene_id::RenderSceneId;
use crate::host::render_scene_update::RenderSceneUpdate;
use crate::host::render_sector_failure::RenderSectorFailure;
use crate::lighting::ambient_gust::AmbientGust;
use crate::scene::level::helpers::level_helpers::LevelHelpers;
use crate::scene::level::level_grass::LevelGrass;
use crate::scene::level::level_lights::LevelLights;
use crate::scene::level::level_load::LevelLoad;
use crate::scene::level::level_particles::LevelParticles;
use crate::scene::level::level_wetness::LevelWetness;
use crate::scene::static_scene::static_item_context::StaticItemContext;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_trace::StaticTrace;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::thread::render_workers::RenderWorkers;

/// A level as the renderer holds it, whatever views draw it: its static geometry, grass, lights and the effects and
/// motions living in it, and the models its weather draws. The world streams it in and owns the files and their
/// failures; the scene owns what is resident on the GPU, reports how far that is ready, and tells the world what it
/// could not take in.
pub struct LevelScene {
  pub source: Arc<dyn RenderLevelSource>,
  pub grass: LevelGrass,
  pub statics: StaticScene,
  /// The static geometry's acceleration structures, while a view of it traces rays on a device that can: tracing is
  /// opted into, and the structures take some 150 MB on Zaton.
  pub trace: Option<StaticTrace>,
  /// The cubes the scene's environment-mapped models mix toward, by environment slot from the second, as of the cache's
  /// generation of them.
  pub environments: (u64, Vec<String>),
  /// When the level began opening on the frames' clock, which the clouds drift and the trees sway from, and the time
  /// this frame is drawn at on it.
  started: Instant,
  now: Instant,
  pub lights: LevelLights,
  pub particles: LevelParticles,
  /// Milliseconds the last sector taken in took to put into the scene.
  pub sector_time: f32,
  /// What its load reports, from how far the world streamed it.
  load: LevelLoad,
  /// Every texture slot it samples, each once, as the last frame left them.
  texture_slots: Vec<u32>,
  /// The viewport driving what it simulates by its camera, as the world named it.
  pub driver: Option<RenderViewportId>,
  /// Every skinned object's bones as segments, which the skeleton overlay draws.
  pub skeleton_segments: Arc<[(Vec3, Vec3)]>,
  /// The level objects' helpers as wires, which every view draws as its options ask.
  pub helpers: LevelHelpers,
  /// The wind the ambient effects blow, and how much of each campfire's idle light shows.
  pub gust: AmbientGust,
  pub campfire_shares: HashMap<u16, f32>,
  /// The driving camera's smoothed hemi, none where it is not estimated.
  pub camera_hemi: Option<f32>,
  /// How wet the level is and how full its puddles, which every view of it wets its surfaces by.
  pub wetness: LevelWetness,
  /// The sectors and the spawned models it could not take in since the world was last told.
  sector_failures: Vec<RenderSectorFailure>,
  model_failures: Vec<RenderModelFailure>,
  /// The sectors the world posted that wait to be taken in, in the order posted, and the frames the first has waited.
  pending: VecDeque<(ProxyHandle<StaticSectorProxy>, SectorPackage)>,
  waited: u32,
}

/// What taking a sector in is named as a job: nothing the timer measures, so its cost stays its seed.
static SECTOR_PASSES: [&str; 1] = ["sector add"];

impl LevelScene {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    source: Arc<dyn RenderLevelSource>,
    workers: &RenderWorkers,
    now: Instant,
  ) -> Self {
    // Taken before anything is made, so the load is timed from the moment the level began opening.
    let opened: Instant = Instant::now();
    let statics: StaticScene = StaticScene::new(device, queue);

    Self {
      grass: LevelGrass::default(),
      lights: LevelLights::new(device, view_layout),
      particles: LevelParticles::new(device, workers),
      statics,
      trace: None,
      environments: (0, Vec::new()),
      started: now,
      now,
      sector_time: 0.0,
      load: LevelLoad::new(opened),
      texture_slots: Vec::new(),
      driver: None,
      skeleton_segments: Arc::from([]),
      helpers: LevelHelpers::default(),
      gust: AmbientGust::default(),
      campfire_shares: HashMap::new(),
      camera_hemi: None,
      wetness: LevelWetness::default(),
      sector_failures: Vec::new(),
      model_failures: Vec::new(),
      pending: VecDeque::new(),
      waited: 0,
      source,
    }
  }

  /// Sectors offered at most a frame once the level stands; milliseconds a byte of one costs to take in (its geometry
  /// written and, traced, its acceleration structure built), and frames one waits at most.
  const SECTORS_OFFERED: usize = 8;
  const SECTOR_SEED: f32 = 1e-7;
  const SECTOR_WAIT_MOST: u32 = 4;

  /// Takes a frame the world gave it: applies what it posted, the sectors as `scheduler` grants them, keeps what its
  /// effects and motions do, moves the lights its motions carry, readies the static draws and, while a view of it
  /// traces rays (`is_traced`), their acceleration structures, and asks for the cubes its surfaces mix toward; the
  /// sectors it could not take in are told back with its feedback.
  pub fn advance(
    &mut self,
    (device, queue, encoder): (&wgpu::Device, &wgpu::Queue, &mut FrameEncoder),
    (textures, weather_textures): (&mut TextureCache, &mut WeatherTextureCache),
    scheduler: &mut PeriodicScheduler,
    (frame, is_traced): (RenderSceneFrame, bool),
  ) {
    let RenderSceneFrame {
      source,
      driver,
      updates,
      streaming,
      skeleton_segments,
      gust,
      campfire_shares,
      motions,
      camera_hemi,
    } = frame;

    // The world moves a scene onto a next source of its level, which every read from now on goes through.
    self.source = source;

    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&self.source) as Arc<dyn RenderAssetSource>;

    if self.environments.0 != textures.get_environments_generation() {
      self.environments = (
        textures.get_environments_generation(),
        textures.list_environments().to_vec(),
      );
    }

    // Only the cubes this scene samples are kept loaded; another scene's are bound as placeholders here.
    for slot in &self.statics.surfaces.environment_slots {
      if let Some(reference) = textures.get_environment(*slot) {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }

    self.apply((device, queue, encoder), (&mut *textures, &assets), updates);

    let failures: Vec<RenderSectorFailure> =
      self.add_sectors((device, queue, encoder), (&mut *textures, &assets), scheduler);

    self.sector_failures.extend(failures);
    // Refilled in place, so a frame allocates nothing once the list is as long as it gets.
    let mut slots: Vec<u32> = std::mem::take(&mut self.texture_slots);

    slots.clear();
    slots.extend(self.list_texture_slots());
    slots.sort_unstable();
    slots.dedup();
    self.texture_slots = slots;
    self.load.advance(streaming, &self.texture_slots, textures);
    self.driver = driver;
    self.skeleton_segments = skeleton_segments;
    self.gust = gust;
    self.campfire_shares = campfire_shares;
    self.camera_hemi = camera_hemi;
    self.lights.begin_frame(&motions);
    self.statics.prepare_draws(device, queue, encoder);

    // Made when a view starts tracing, let go when none does.
    if !(is_traced && self.statics.is_traced()) {
      self.trace = None;
    } else if self.trace.is_none() {
      self.trace = Some(StaticTrace::new(device, queue));
    }

    if let Some(trace) = &mut self.trace {
      trace.update(device, queue, encoder, &self.statics);
    }
  }

  /// Moves it to the time this frame is drawn at on the frames' clock, which everything living in it moves by.
  pub fn set_now(&mut self, now: Instant) {
    self.now = now;
  }

  /// The time this frame is drawn at, on the frames' clock.
  pub fn get_now(&self) -> Instant {
    self.now
  }

  /// How long it has lived at this frame, on the frames' clock: what its clouds, sway, water, lights and effects move
  /// by.
  pub fn get_time(&self) -> Duration {
    self.now.saturating_duration_since(self.started)
  }

  /// The time on the frames' clock that stands `time` after it began opening.
  pub fn get_instant(&self, time: Duration) -> Instant {
    self.started + time
  }

  /// What it tells the world of the frames since the last: the sectors and the spawned models it could not take in,
  /// and the effects whose particles stopped playing.
  pub fn take_feedback(&mut self) -> RenderSceneFeedback {
    RenderSceneFeedback {
      sector_failures: std::mem::take(&mut self.sector_failures),
      model_failures: std::mem::take(&mut self.model_failures),
      finished_effects: self.particles.take_finished(),
    }
  }

  /// How far the level, the scene `id`, has loaded.
  pub fn describe_load(&self, id: RenderSceneId, textures: &TextureCache) -> RenderLoadReport {
    self.load.describe(id, &self.texture_slots, textures)
  }

  /// Whether everything the level opens with is resident, so it draws as it will.
  pub fn is_ready(&self, textures: &TextureCache) -> bool {
    self.load.is_ready(self.list_texture_slots(), textures)
  }

  /// What the level still waits for before it is ready, said for a log.
  pub fn describe_pending(&self, textures: &TextureCache) -> String {
    self.load.describe_pending(&self.texture_slots, textures)
  }

  /// Takes in the sectors waiting as `scheduler` grants them, in a round of their own: every one while the level opens,
  /// so it opens as fast as it streams; once it stands, the first `SECTORS_OFFERED` by their bytes, each waiting at
  /// most `SECTOR_WAIT_MOST` frames. Answers those it could not take in.
  fn add_sectors(
    &mut self,
    (device, queue, encoder): (&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder),
    (textures, assets): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
    scheduler: &mut PeriodicScheduler,
  ) -> Vec<RenderSectorFailure> {
    if self.pending.is_empty() {
      return Vec::new();
    }

    let is_opening: bool = !self.is_ready(textures);
    let count: usize = if is_opening {
      self.pending.len()
    } else {
      self.pending.len().min(Self::SECTORS_OFFERED)
    };
    let tickets: Vec<PeriodicTicket> = self
      .pending
      .iter()
      .take(count)
      .map(|(_, package)| {
        let bytes: f32 = package.buffer.len() as f32;

        scheduler.offer(if is_opening {
          PeriodicJob::new_now(&SECTOR_PASSES, Self::SECTOR_SEED, bytes)
        } else {
          PeriodicJob::new_waiting(
            &SECTOR_PASSES,
            Self::SECTOR_SEED,
            bytes,
            (self.waited, Self::SECTOR_WAIT_MOST),
          )
        })
      })
      .collect();

    scheduler.grant();

    let offered: Vec<(ProxyHandle<StaticSectorProxy>, SectorPackage)> = self.pending.drain(..count).collect();
    let mut failures: Vec<RenderSectorFailure> = Vec::new();
    let mut kept: Vec<(ProxyHandle<StaticSectorProxy>, SectorPackage)> = Vec::new();

    for ((handle, package), ticket) in offered.into_iter().zip(tickets) {
      if !scheduler.is_granted(ticket) {
        kept.push((handle, package));

        continue;
      }

      let started: Instant = Instant::now();
      let added: XrfResult = self.statics.add_sector(
        &mut StaticItemContext {
          device,
          queue,
          encoder: &mut *encoder,
          textures: &mut *textures,
          source: assets,
        },
        self.source.get_surfaces(),
        (handle, &package),
      );

      self.sector_time = started.elapsed().as_secs_f32() * 1000.0;

      if let Err(error) = added {
        failures.push(RenderSectorFailure {
          handle,
          reason: error.to_string(),
        });
      }
    }

    self.waited = if kept.is_empty() { 0 } else { self.waited + 1 };

    for item in kept.into_iter().rev() {
      self.pending.push_front(item);
    }

    failures
  }

  /// Applies what the world posted this frame, in the order posted, the sectors queued to be taken in.
  fn apply(
    &mut self,
    (device, queue, encoder): (&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder),
    (textures, assets): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
    updates: Vec<RenderSceneUpdate>,
  ) {
    for update in updates {
      match update {
        RenderSceneUpdate::AddSector { handle, package } => self.pending.push_back((handle, package)),
        // One still waiting is never taken in.
        RenderSceneUpdate::RemoveSector(handle) => match self.pending.iter().position(|(it, _)| *it == handle) {
          Some(index) => {
            self.pending.remove(index);
          }
          None => {
            self.statics.remove_sector(handle);
          }
        },
        RenderSceneUpdate::AddModel { handle, model } => {
          if let Err(error) = self.statics.add_model(
            &mut StaticItemContext {
              device,
              queue,
              encoder: &mut *encoder,
              textures: &mut *textures,
              source: assets,
            },
            (handle, &model),
          ) {
            log::warn!("Native viewport could not add a spawned model: {error}");

            self.model_failures.push(RenderModelFailure {
              handle,
              reason: error.to_string(),
            });
          }
        }
        RenderSceneUpdate::RemoveModel(handle) => {
          self.statics.remove_model(handle);
        }
        // The objects of a model refused are told back with it: the world takes them out.
        RenderSceneUpdate::AddObject { model, .. }
          if self.model_failures.iter().any(|failure| failure.handle == model) => {}
        RenderSceneUpdate::AddObject { handle, model, place } => {
          if let Err(error) = self.statics.add_object(device, queue, handle, (model, &place)) {
            log::warn!(
              "Native viewport could not stand spawned object {}: {error}",
              place.object
            );
          }
        }
        RenderSceneUpdate::RemoveObject(handle) => {
          self.statics.remove_object(handle);
        }
        RenderSceneUpdate::AddLights(lights) => self.lights.add_lights(lights, textures, assets),
        RenderSceneUpdate::AddGrass(details) => self.grass.replace(device, details, (&mut *textures, assets)),
        RenderSceneUpdate::SetHelpers(helpers) => {
          self.helpers = LevelHelpers::new(helpers, self.helpers.version + 1);
        }
        RenderSceneUpdate::AddParticles(definitions) => {
          self
            .particles
            .add_definitions(device, (&mut *textures, assets), definitions);
        }
        RenderSceneUpdate::AddEmitter {
          handle,
          transform,
          seed,
        } => self.particles.add_emitter(handle, transform, seed),
        RenderSceneUpdate::RemoveEmitter(handle) => self.particles.remove_emitter(handle),
        RenderSceneUpdate::PlayEffect { handle, effect, name } => {
          let time: Duration = self.get_time();

          self.particles.play(handle, effect, &name, time);
        }
        RenderSceneUpdate::StopEffect {
          handle,
          effect,
          is_deferred,
        } => self.particles.stop(handle, effect, is_deferred),
        RenderSceneUpdate::MoveEmitter {
          handle,
          transform,
          velocity,
        } => self.particles.move_emitter(handle, transform, velocity),
        RenderSceneUpdate::CarryEffect {
          handle,
          effect,
          velocity,
        } => self.particles.carry(handle, effect, velocity),
        RenderSceneUpdate::PoseObject {
          handle,
          current,
          previous,
        } => self.statics.write_pose(handle, &current, &previous),
      }
    }
  }

  /// What became of every texture the level's surfaces sample, then each the grass samples that none of them does.
  pub fn describe_textures(&self, textures: &TextureCache) -> Vec<RenderTextureReport> {
    let surfaces: BTreeSet<u32> = self.statics.surfaces.texture_slots.iter().copied().collect();
    let grass: BTreeSet<u32> = self
      .grass
      .list_texture_slots()
      .filter(|slot| !surfaces.contains(slot))
      .collect();

    let mut described: Vec<RenderTextureReport> = textures.describe(&surfaces, RenderTextureGroup::Surfaces);

    described.extend(textures.describe(&grass, RenderTextureGroup::Grass));
    described
  }

  /// A spawned object's bounding sphere in renderer space, once its model is in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.statics.get_object_sphere(object)
  }

  /// Every environment slot it samples, so the cubes no scene samples can be freed.
  pub fn list_environment_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self.statics.surfaces.environment_slots.iter().copied()
  }

  /// Every texture slot it samples, once for each of its parts sampling it, so the slots no scene samples can be freed.
  pub fn list_texture_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self
      .statics
      .surfaces
      .texture_slots
      .iter()
      .copied()
      .chain(self.grass.list_texture_slots())
      .chain(self.lights.get_projectors().iter().copied())
      .chain(self.particles.get_texture_slots().iter().copied())
  }
}
