use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use glam::{Vec3, Vec4};
use xrf_error::XrfResult;

use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_viewport_id::RenderViewportId;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_scene_feedback::RenderSceneFeedback;
use crate::host::render_scene_frame::RenderSceneFrame;
use crate::host::render_scene_update::RenderSceneUpdate;
use crate::host::render_sector_failure::RenderSectorFailure;
use crate::lighting::ambient_gust::AmbientGust;
use crate::pass::grass_pass::GrassPass;
use crate::scene::level::level_grass::LevelGrass;
use crate::scene::level::level_lights::LevelLights;
use crate::scene::level::level_load::LevelLoad;
use crate::scene::level::level_particles::LevelParticles;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::thread::render_workers::RenderWorkers;

/// A level as the renderer holds it, whatever views draw it: its static geometry, grass, lights and the effects and
/// motions living in it, and the models its weather draws. Its streaming fills it; it knows nothing of files, failures
/// or readiness.
pub struct LevelScene {
  pub source: Arc<dyn RenderLevelSource>,
  pub grass: LevelGrass,
  pub statics: StaticScene,
  /// The cubes the scene's environment-mapped models mix toward, by environment slot from the second, as of the cache's
  /// generation of them.
  pub environments: (u64, Vec<String>),
  /// When the level began opening, which the clouds drift and the trees sway from.
  pub started: Instant,
  pub lights: LevelLights,
  pub particles: LevelParticles,
  /// Milliseconds the last sector taken in took to put into the scene.
  pub sector_time: f32,
  /// What its load reports, from how far the world streamed it.
  load: LevelLoad,
  /// The viewport driving what it simulates by its camera, as the world named it.
  pub driver: Option<RenderViewportId>,
  /// Every skinned object's bones as segments, which the skeleton overlay draws.
  pub skeleton_segments: Vec<(Vec3, Vec3)>,
  /// The wind the ambient effects blow, and how much of each campfire's idle light shows.
  pub gust: AmbientGust,
  pub campfire_shares: HashMap<u16, f32>,
  /// The driving camera's smoothed hemi, none where it is not estimated.
  pub camera_hemi: Option<f32>,
  /// The sectors it could not take in since the world was last told.
  failures: Vec<RenderSectorFailure>,
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
      grass: LevelGrass::new(&source, workers),
      lights: LevelLights::new(device, view_layout, statics.args.size()),
      particles: LevelParticles::new(device, workers),
      statics,
      environments: (0, Vec::new()),
      started,
      sector_time: 0.0,
      load: LevelLoad::new(started),
      driver: None,
      skeleton_segments: Vec::new(),
      gust: AmbientGust::default(),
      campfire_shares: HashMap::new(),
      camera_hemi: None,
      failures: Vec::new(),
      source,
    }
  }

  /// Takes a frame the world gave it: applies what it posted, takes the grass its loader read, keeps what its effects
  /// and motions do, moves the lights its motions carry, readies the static draws, and asks for the cubes its surfaces
  /// mix toward; the sectors it could not take in are told back with its feedback.
  pub fn advance(
    &mut self,
    (device, queue, encoder): (&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder),
    (textures, weather_textures): (&mut TextureCache, &mut WeatherTextureCache),
    grass_pass: &GrassPass,
    frame: RenderSceneFrame,
  ) {
    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&self.source) as Arc<dyn RenderAssetSource>;

    if self.environments.0 != textures.get_environments_generation() {
      self.environments = (
        textures.get_environments_generation(),
        textures.list_environments().to_vec(),
      );
    }

    // Only the cubes this scene samples are kept loaded; another scene's are bound as placeholders here.
    for slot in &self.statics.environment_slots {
      if let Some(reference) = textures.get_environment(*slot) {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }

    if let Some(slots) = self.grass.poll(device, grass_pass, textures, &assets) {
      self.statics.texture_slots.extend(slots);
    }

    let RenderSceneFrame {
      driver,
      updates,
      streaming,
      skeleton_segments,
      gust,
      campfire_shares,
      motions,
      camera_hemi,
      ..
    } = frame;
    let failures: Vec<RenderSectorFailure> = self.apply((device, queue, encoder), (&mut *textures, &assets), updates);

    self.failures.extend(failures);
    let slots: Vec<u32> = self.list_texture_slots().collect();

    self.load.advance(streaming, (self.grass.is_loaded(), &slots), textures);
    self.driver = driver;
    self.skeleton_segments = skeleton_segments;
    self.gust = gust;
    self.campfire_shares = campfire_shares;
    self.camera_hemi = camera_hemi;
    self.lights.begin_frame(&motions);
    self.statics.prepare_draws(device, queue, encoder);
  }

  /// What it tells the world of the frames since the last: the sectors it could not take in, and the effects whose
  /// particles stopped playing.
  pub fn take_feedback(&mut self) -> RenderSceneFeedback {
    RenderSceneFeedback {
      failures: std::mem::take(&mut self.failures),
      finished_effects: self.particles.take_finished(),
    }
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_load_report(&mut self, textures: &TextureCache) -> Option<RenderLoadReport> {
    let slots: Vec<u32> = self.list_texture_slots().collect();

    self.load.take_report((self.grass.is_loaded(), &slots), textures)
  }

  /// How far the level has loaded.
  pub fn describe_load(&self, textures: &TextureCache) -> RenderLoadReport {
    let slots: Vec<u32> = self.list_texture_slots().collect();

    self.load.describe((self.grass.is_loaded(), &slots), textures)
  }

  /// Whether everything the level opens with is resident, so it draws as it will.
  pub fn is_ready(&self, textures: &TextureCache) -> bool {
    self.describe_load(textures).is_ready
  }

  /// Applies what the world posted this frame, in the order posted; answers the sectors it could not take in.
  fn apply(
    &mut self,
    (device, queue, encoder): (&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder),
    (textures, assets): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
    updates: Vec<RenderSceneUpdate>,
  ) -> Vec<RenderSectorFailure> {
    let mut failures: Vec<RenderSectorFailure> = Vec::new();

    for update in updates {
      match update {
        RenderSceneUpdate::AddSector { handle, package } => {
          let started: Instant = Instant::now();
          let added: XrfResult = self.statics.add_sector(
            device,
            queue,
            encoder,
            textures,
            assets,
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
        RenderSceneUpdate::RemoveSector(handle) => {
          self.statics.remove_sector(handle);
        }
        RenderSceneUpdate::AddModel { handle, model } => {
          if let Err(error) = self
            .statics
            .add_model(device, queue, encoder, (&mut *textures, assets), (handle, &model))
          {
            log::warn!("Native viewport could not add a spawned model: {error}");
          }
        }
        RenderSceneUpdate::RemoveModel(handle) => {
          self.statics.remove_model(handle);
        }
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
        RenderSceneUpdate::PlayEffect { handle, effect, name } => self.particles.play(handle, effect, &name),
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

    failures
  }

  /// What became of every texture the level's surfaces sample.
  pub fn describe_textures(&self, textures: &TextureCache) -> Vec<RenderTextureReport> {
    textures.describe(&self.statics.texture_slots)
  }

  /// A spawned object's bounding sphere in renderer space, once its model is in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.statics.get_object_sphere(object)
  }

  /// Every environment slot it samples, so the cubes no scene samples can be freed.
  pub fn list_environment_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self.statics.environment_slots.iter().copied()
  }

  /// Every texture slot it samples, so the slots no scene samples can be freed.
  pub fn list_texture_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self
      .statics
      .texture_slots
      .iter()
      .copied()
      .chain(self.lights.get_projectors().iter().copied())
      .chain(self.particles.get_texture_slots().iter().copied())
  }

  /// Bytes its scene's growing buffers hold on the GPU.
  pub fn get_buffer_bytes(&self) -> u64 {
    self.statics.get_buffer_bytes()
  }

  /// Whether it draws this source.
  pub fn is_showing(&self, source: &Arc<dyn RenderLevelSource>) -> bool {
    Arc::ptr_eq(&self.source, source)
  }
}
