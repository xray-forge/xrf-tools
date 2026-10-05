use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec3, Vec4};

use crate::contract::render_ambient_report::RenderAmbientReport;
use crate::contract::render_model_pose::RenderModelPose;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_motion::RenderMotion;
use crate::scene::level::level_campfires::LevelCampfires;
use crate::scene::level::level_grass::LevelGrass;
use crate::scene::level::level_lights::LevelLights;
use crate::scene::level::level_object_motions::LevelObjectMotions;
use crate::scene::level::level_particles::LevelParticles;
use crate::scene::level::model_motions::ModelMotions;
use crate::scene::level::posed_skeleton::PosedSkeleton;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;
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
  /// The splash's model, with the level's weather it was built for.
  pub splash: Option<(usize, WeatherModelBuffers)>,
  /// Every bolt model of the level's weather, with the weather they were built for, and an empty one the glows bind.
  pub thunder_models: Option<(usize, Vec<WeatherModelBuffers>)>,
  pub no_model: WeatherModelBuffers,
  /// When the level began opening, which the clouds drift and the trees sway from.
  pub started: Instant,
  pub lights: LevelLights,
  pub campfires: LevelCampfires,
  /// The object motions its moving zones follow, which their particles and lights both read.
  pub object_motions: LevelObjectMotions,
  pub particles: LevelParticles,
  /// Each skinned object's skeleton, by its index, the motions they are posed by, and the pose asked for.
  pub skeletons: HashMap<u32, PosedSkeleton>,
  pub motions: ModelMotions,
  pub model_pose: RenderModelPose,
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
      grass: LevelGrass::new(device, &source, workers),
      lights: LevelLights::new(device, view_layout, statics.args.size()),
      campfires: LevelCampfires::new(),
      object_motions: LevelObjectMotions::new(&source, workers),
      particles: LevelParticles::new(device, &source, workers),
      statics,
      environments: (0, Vec::new()),
      splash: None,
      thunder_models: None,
      no_model: WeatherModelBuffers::new(device, None),
      started,
      skeletons: HashMap::new(),
      motions: ModelMotions::new(workers),
      model_pose: RenderModelPose::default(),
      source,
    }
  }

  /// What became of every texture the level's surfaces sample.
  pub fn describe_textures(&self, textures: &TextureCache) -> Vec<RenderTextureReport> {
    textures.describe(&self.statics.texture_slots)
  }

  /// Stands every skinned object as asked from the next frame on.
  pub fn set_model_pose(&mut self, pose: &RenderModelPose) {
    if self.model_pose != *pose {
      self.model_pose = pose.clone();
    }
  }

  /// Writes every standing skinned object's bone matrices for this frame, and the last frame's beside them; a motion
  /// still on its way poses the bind pose meanwhile.
  pub fn pose_skeletons(&mut self) {
    if self.skeletons.is_empty() {
      return;
    }

    let pose: &RenderModelPose = &self.model_pose;
    let motion: Option<&RenderMotion> = match &pose.motion {
      Some(name) => self.motions.get(&self.source, name),
      None => None,
    };

    for (object, skeleton) in &mut self.skeletons {
      if !self.statics.has_object(*object) {
        continue;
      }

      let (current, previous) = skeleton.pose(motion, pose.frame, &pose.hidden_bones);

      self.statics.write_pose(*object, &current, &previous);
    }
  }

  /// Every skinned object's bones as segments in renderer space, child then parent, where this frame poses them.
  pub fn list_skeleton_segments(&self) -> Vec<(Vec3, Vec3)> {
    self
      .skeletons
      .iter()
      .flat_map(|(object, skeleton)| {
        let place: Mat4 = self.statics.get_object_transform(*object).unwrap_or(Mat4::IDENTITY);

        skeleton
          .list_segments()
          .into_iter()
          .map(move |(child, parent)| (place.transform_point3(child), place.transform_point3(parent)))
      })
      .collect()
  }

  /// A spawned object's bounding sphere in renderer space, once its model is in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.statics.get_object_sphere(object)
  }

  /// Plays a weather ambient effect on the next frame, without waiting.
  pub fn play_ambient_now(&mut self) {
    self.particles.play_ambient_now();
  }

  /// Where the weather's ambient effects near the camera stand, none until the particles are read.
  pub fn get_ambient_report(&self) -> Option<RenderAmbientReport> {
    self.particles.get_ambient_report()
  }

  /// What the level's particle systems came to since the last report.
  pub fn take_particles_report(&mut self) -> RenderParticlesReport {
    self.particles.take_report()
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
