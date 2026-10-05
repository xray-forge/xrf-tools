use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec3, Vec4};
use rayon::prelude::*;
use xrf_engine_target::XrayEngine;
use xrf_material::{XraySurfaceDraw, XraySurfaceSampler};
use xrf_particles::{
  ParticleBounds, ParticleCollider, ParticleEffectInstance, ParticleEngineRules, ParticleLibrary, ParticleObject,
  ParticleUpdateContext,
};
use xrf_renderer_core::{FrameGraph, GraphColorAttachment, GraphDepthAttachment};
use xrf_visual::ZoneSphere;

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_report::RenderAmbientReport;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_particles::RenderLevelParticles;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_particle_source::RenderParticleSource;
use crate::pass::particle_batch::ParticleBatch;
use crate::pass::particle_blend::ParticleBlend;
use crate::pass::particle_pass::ParticlePass;
use crate::pass::particle_surface_record::ParticleSurfaceRecord;
use crate::pass::particle_vertex::ParticleVertex;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::ambient_frame::AmbientFrame;
use crate::scene::level::ambient_gust::AmbientGust;
use crate::scene::level::camera_hemi::CameraHemi;
use crate::scene::level::campfire::Campfire;
use crate::scene::level::level_ambient_effects::LevelAmbientEffects;
use crate::scene::level::level_campfires::LevelCampfires;
use crate::scene::level::level_object_motions::LevelObjectMotions;
use crate::scene::level::loader_answer::take_answer;
use crate::scene::level::particle_sprite::ParticleSprite;
use crate::scene::level::placed_effect::PlacedEffect;
use crate::scene::level::zone_fast_mode::ZONE_FAST_DISTANCE;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;
use crate::thread::loader_receiver::LoaderReceiver;
use crate::thread::render_workers::RenderWorkers;

/// Quads the vertex buffer holds at first; it doubles past them.
const INITIAL_QUADS: u64 = 4096;

/// The sampler a distorting effect's `l_special` pass binds its distortion map to.
const DISTORTION_SAMPLER: &str = "s_distort";

/// A level's particle systems and the weather's ambient effects: read once on a loader thread, then each frame stepped
/// on the workers as the engine schedules them, and the effects in view filled into quads far to near for the particle
/// pass, which draws their colour and then their distortion.
pub struct LevelParticles {
  pending: Option<LoaderReceiver<Result<Option<RenderLevelParticles>, String>>>,
  systems: Option<LevelSystems>,
  workers: RenderWorkers,
  started: Instant,
  vertices: Vec<ParticleVertex>,
  batches: Vec<ParticleBatch>,
  /// The distorting effects' quads, far to near.
  distortion_runs: Vec<Range<u32>>,
  vertex_buffer: wgpu::Buffer,
  surface_buffer: wgpu::Buffer,
  /// Bumped whenever either buffer is replaced, which the bind group follows.
  buffers_generation: u64,
  /// Whether the surfaces' records still have to be written.
  is_surfaces_dirty: bool,
  /// The bind group, with the targets' epoch and the buffers' generation it binds.
  group: Option<((u64, u64), wgpu::BindGroup)>,
  report: ParticlesTally,
}

/// What the loader read, placed and stepping.
struct LevelSystems {
  library: Arc<ParticleLibrary>,
  rules: ParticleEngineRules,
  collider: Option<Arc<dyn ParticleCollider>>,
  systems: Vec<PlacedSystem>,
  ambient: LevelAmbientEffects,
  hemi: CameraHemi,
  surfaces: Vec<ParticleSurface>,
  /// Each effect definition's surface, by the definition's address.
  surface_of: HashMap<usize, u32>,
  texture_slots: Vec<u32>,
}

/// One placement, what plays at it, where it stands this frame, and for a campfire whether it was lit when last placed.
struct PlacedSystem {
  source: RenderParticleSource,
  transform: Mat4,
  /// The object motion carrying its zone, which moves the transform each frame.
  motion: Option<String>,
  /// Its zone's sphere, offset from the transform's place, which Monolith measures how far the camera stands by.
  zone_sphere: Option<ZoneSphere>,
  seed: i32,
  campfire_lit: Option<bool>,
  objects: PlacedObjects,
}

/// What plays at a placed system, an object a [`PlacedEffect`].
#[derive(Default)]
struct PlacedObjects([Option<ParticleObject>; PlacedEffect::COUNT]);

/// How an effect's sprite draws: its equation, none for one drawing no colour, whether it distorts, and its record.
struct ParticleSurface {
  blend: Option<ParticleBlend>,
  is_distorting: bool,
  record: ParticleSurfaceRecord,
}

/// What the frames since the last report came to.
#[derive(Default)]
struct ParticlesTally {
  last: RenderParticlesReport,
  simulation_time: f32,
  frames: u32,
}

impl LevelParticles {
  pub fn new(device: &wgpu::Device, source: &Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();
    let source: Arc<dyn RenderLevelSource> = Arc::clone(source);

    workers.spawn(move || {
      let particles = source.read_particles().map_err(|error| error.to_string());

      if let Err(error) = &particles {
        log::error!("The level's particles cannot be drawn: {error}");
      }

      let _ = sender.send(particles);
    });

    Self {
      pending: Some(receiver),
      systems: None,
      workers: workers.clone(),
      started: Instant::now(),
      vertices: Vec::new(),
      batches: Vec::new(),
      distortion_runs: Vec::new(),
      vertex_buffer: Self::create_storage(
        device,
        "particle vertices",
        INITIAL_QUADS * u64::from(ParticleVertex::CORNERS) * size_of::<ParticleVertex>() as u64,
      ),
      surface_buffer: Self::create_storage(device, "particle surfaces", size_of::<ParticleSurfaceRecord>() as u64),
      buffers_generation: 0,
      is_surfaces_dirty: false,
      group: None,
      report: ParticlesTally::default(),
    }
  }

  /// Takes the systems once their loader read them: asks for every sprite's textures and places each system.
  pub fn poll(&mut self, device: &wgpu::Device, textures: &mut TextureCache, source: &Arc<dyn RenderAssetSource>) {
    let Some(read) = take_answer(&mut self.pending, "particles") else {
      return;
    };
    let Ok(Some(read)) = read else {
      return;
    };
    let mut surfaces: Vec<ParticleSurface> = Vec::with_capacity(read.surfaces.len());
    let mut surface_of: HashMap<usize, u32> = HashMap::with_capacity(read.surfaces.len());
    let mut texture_slots: Vec<u32> = Vec::new();

    for (name, descriptor) in &read.surfaces {
      let Some(effect) = read.library.get_effect(name) else {
        continue;
      };
      let blend: Option<ParticleBlend> = ParticleBlend::of(descriptor.draw);
      let texture: Option<&str> = descriptor
        .textures
        .first()
        .map(String::as_str)
        .filter(|_| blend.is_some());
      let distortion: Option<&str> =
        descriptor.find_sampler(XraySurfaceSampler::DISTORTION_ELEMENT, DISTORTION_SAMPLER);
      let mut request = |reference: Option<&str>, role: TextureRole| {
        reference.map_or(0, |reference| {
          let slot: u32 = textures.request(reference, role, source);

          texture_slots.push(slot);

          slot
        })
      };

      surface_of.insert(Arc::as_ptr(effect) as usize, surfaces.len() as u32);
      surfaces.push(ParticleSurface {
        record: ParticleSurfaceRecord {
          texture: request(texture, TextureRole::Base),
          flags: if descriptor.is_texture_clamped {
            ParticleSurfaceRecord::IS_CLAMPED
          } else {
            0
          },
          // Every forward particle pass tests against zero; `SET` against its own reference.
          alpha_reference: match descriptor.draw {
            XraySurfaceDraw::AlphaTested { reference } => f32::from(reference) / 255.0,
            _ => 0.0,
          },
          // As the water's own distortion map, neutral at mid grey while it loads.
          distortion: request(distortion, TextureRole::Detail),
        },
        is_distorting: distortion.is_some(),
        blend,
      });
    }

    let systems: Vec<PlacedSystem> = read
      .placements
      .iter()
      .enumerate()
      .map(|(index, placement)| PlacedSystem {
        source: placement.source.clone(),
        transform: Mat4::from_cols_array(&placement.transform),
        motion: placement.motion.clone(),
        zone_sphere: placement.zone_sphere.clone(),
        seed: index as i32 + 1,
        campfire_lit: None,
        objects: PlacedObjects::default(),
      })
      .collect();

    log::info!(
      "Native viewport particles {} placed systems, {} effect surfaces",
      systems.len(),
      surfaces.len()
    );

    self.surface_buffer = Self::create_storage(
      device,
      "particle surfaces",
      (surfaces.len().max(1) * size_of::<ParticleSurfaceRecord>()) as u64,
    );
    self.buffers_generation += 1;
    self.is_surfaces_dirty = true;
    self.systems = Some(LevelSystems {
      library: read.library,
      rules: read.rules,
      collider: read.collider,
      systems,
      ambient: LevelAmbientEffects::default(),
      hemi: CameraHemi::new(read.hemi, &self.workers),
      surfaces,
      surface_of,
      texture_slots,
    });
  }

  /// Plays the weather's ambient effects by the frame's weather and blows the wind they bring, before anything reads
  /// the wind this frame; frozen while particles are not drawn, and played as though indoors while switched off.
  pub fn update_ambient(&mut self, view: &CameraView, options: &RenderViewOptions, ambient: Option<AmbientFrame<'_>>) {
    let Some(level) = self.systems.as_mut() else {
      return;
    };

    if !options.show.is_particled || !options.mode.is_lit {
      return;
    }

    let now: u64 = self.started.elapsed().as_millis() as u64;
    let eye: Vec3 = ParticleSprite::mirror(view.position);
    let context: ParticleUpdateContext = ParticleUpdateContext {
      library: &level.library,
      rules: &level.rules,
      collider: level.collider.as_deref(),
    };

    level.hemi.advance(eye, now);
    level.ambient.update(
      ambient,
      (eye, level.hemi.is_indoors() || !options.world.is_ambient_played),
      now,
      &context,
    );
  }

  /// Plays an ambient effect on the next frame, ending the one playing, without waiting.
  pub fn play_ambient_now(&mut self) {
    if let Some(level) = &mut self.systems {
      level.ambient.play_now();
    }
  }

  /// Where the ambient effects stand, none until the particles are read.
  pub fn get_ambient_report(&self) -> Option<RenderAmbientReport> {
    let now: u64 = self.started.elapsed().as_millis() as u64;

    self
      .systems
      .as_ref()
      .map(|level| level.ambient.report(now, level.hemi.is_indoors()))
  }

  /// The wind as the ambient effects blow it this frame, still air until the particles are read.
  pub fn get_gust(&self) -> AmbientGust {
    self
      .systems
      .as_ref()
      .map_or_else(AmbientGust::default, |level| level.ambient.get_gust())
  }

  /// Steps every system and the ambient effect playing as the engine schedules them, and fills the effects in view into
  /// quads, far to near.
  pub fn step(
    &mut self,
    view: &CameraView,
    options: &RenderViewOptions,
    campfires: &mut LevelCampfires,
    motions: &mut LevelObjectMotions,
  ) {
    self.vertices.clear();
    self.batches.clear();
    self.distortion_runs.clear();

    let Some(level) = self.systems.as_mut() else {
      return;
    };

    if !options.show.is_particled || !options.mode.is_lit {
      self.report.last = RenderParticlesReport::default();

      return;
    }

    let now: u64 = self.started.elapsed().as_millis() as u64;
    let eye: Vec3 = ParticleSprite::mirror(view.position);
    let planes: [Vec4; 6] = view.get_planes();
    let started: Instant = Instant::now();

    Self::move_systems(level, motions);
    Self::place(level, eye, now, campfires);

    let context: ParticleUpdateContext = ParticleUpdateContext {
      library: &level.library,
      rules: &level.rules,
      collider: level.collider.as_deref(),
    };
    let systems: &mut Vec<PlacedSystem> = &mut level.systems;
    let mut simulated: u32 = self.workers.install(|| {
      systems
        .par_iter_mut()
        .flat_map_iter(|system| system.objects.iter_mut())
        .map(|object| {
          let is_in_view: bool = Self::is_in_view(&planes, object.get_instance().get_bounds());

          u32::from(object.advance(now, eye, is_in_view, &context))
        })
        .sum()
    });

    if let Some(object) = level.ambient.get_playing_mut() {
      let is_in_view: bool = Self::is_in_view(&planes, object.get_instance().get_bounds());

      simulated += u32::from(object.advance(now, eye, is_in_view, &context));
    }
    let simulation_time: f32 = started.elapsed().as_secs_f32() * 1000.0;
    let mut drawn: Vec<(f32, &ParticleEffectInstance, u32, &ParticleSurface)> = Vec::new();
    let mut report: RenderParticlesReport = RenderParticlesReport {
      simulated,
      ..RenderParticlesReport::default()
    };

    for object in level
      .systems
      .iter()
      .flat_map(|system| system.objects.iter())
      .chain(level.ambient.get_playing())
    {
      for effect in object.get_instance().get_effects() {
        let count: usize = effect.get_pool().len();

        report.effects += u32::from(effect.is_playing());
        report.particles += count as u32;

        let Some(&surface) = level
          .surface_of
          .get(&(std::ptr::from_ref(effect.get_definition()) as usize))
        else {
          continue;
        };

        let drawing: &ParticleSurface = &level.surfaces[surface as usize];
        let is_drawing: bool = drawing.blend.is_some() || drawing.is_distorting;

        if is_drawing && count > 0 && Self::is_in_view(&planes, effect.get_bounds()) {
          let (center, _) = effect.get_bounds().get_sphere();

          drawn.push((center.distance_squared(eye), effect, surface, drawing));
        }
      }
    }

    // Far to near, as `mapSorted` draws what blends and `mapDistort` what distorts; every effect is sorted, as nothing
    // else orders them.
    drawn.sort_by(|a, b| b.0.total_cmp(&a.0));
    report.drawn = drawn.len() as u32;

    let sprite: ParticleSprite = ParticleSprite::new(view.view);

    for (_, effect, surface, drawing) in drawn {
      let first: u32 = self.vertices.len() as u32 / ParticleVertex::CORNERS;

      sprite.push(effect, surface, &mut self.vertices);

      let end: u32 = self.vertices.len() as u32 / ParticleVertex::CORNERS;

      if let Some(blend) = drawing.blend {
        match self.batches.last_mut() {
          Some(last) if last.blend == blend && last.first + last.count == first => last.count = end - last.first,
          _ => self.batches.push(ParticleBatch {
            blend,
            first,
            count: end - first,
          }),
        }
      }

      if drawing.is_distorting {
        match self.distortion_runs.last_mut() {
          Some(last) if last.end == first => last.end = end,
          _ => self.distortion_runs.push(first..end),
        }
      }
    }

    self.report.last = report;
    self.report.simulation_time += simulation_time;
    self.report.frames += 1;
  }

  /// Writes the quads the last step filled, and the surfaces once they are read, and binds what the pass draws them
  /// with.
  pub fn upload(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pass: &ParticlePass,
    lighting: &wgpu::Buffer,
    (targets, targets_epoch): (&ViewTargets, u64),
  ) {
    self.write_buffers(device, queue);

    let key: (u64, u64) = (targets_epoch, self.buffers_generation);

    if self.group.as_ref().is_none_or(|(bound, _)| *bound != key) {
      self.group = Some((
        key,
        pass.create_bind_group(device, &self.vertex_buffer, &self.surface_buffer, lighting, targets),
      ));
    }
  }

  /// Draws the quads filled this frame over the scene, then the distorting ones into the distortion target; whether
  /// anything drew.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    pass: &'a ParticlePass,
    targets: ViewTargetHandles,
    (view, texture_group): (&'a ViewBinding, &'a wgpu::BindGroup),
  ) {
    let Some((_, group)) = &self.group else {
      return;
    };

    if !self.batches.is_empty() {
      graph
        .add_raster_pass("particles")
        .color(GraphColorAttachment::new(targets.scene, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| pass.record_colour(context.get_pass(), (view, group, texture_group), &self.batches));
    }

    if self.is_distorting() {
      graph
        .add_raster_pass("particle distortion")
        .color(GraphColorAttachment::new(targets.distortion, wgpu::LoadOp::Load))
        .depth(GraphDepthAttachment::new_read_only(targets.depth))
        .record(move |context| {
          pass.record_distortion(context.get_pass(), (view, group, texture_group), &self.distortion_runs)
        });
    }
  }

  /// Whether this frame has particles to draw, as `record` would draw them.
  pub fn is_drawing(&self) -> bool {
    self.group.is_some() && (!self.batches.is_empty() || self.is_distorting())
  }

  /// Whether this frame's particles draw into the distortion target.
  pub fn is_distorting(&self) -> bool {
    !self.distortion_runs.is_empty()
  }

  /// Whether its loader has answered, whatever it answered.
  pub fn is_loaded(&self) -> bool {
    self.pending.is_none()
  }

  /// The texture slots the sprites sample.
  pub fn get_texture_slots(&self) -> &[u32] {
    self
      .systems
      .as_ref()
      .map_or(&[], |systems| systems.texture_slots.as_slice())
  }

  /// What the last frame came to, with the simulation's mean cost since the last report.
  pub fn take_report(&mut self) -> RenderParticlesReport {
    let report: RenderParticlesReport = RenderParticlesReport {
      simulation_time: self.report.simulation_time / self.report.frames.max(1) as f32,
      ..self.report.last
    };

    self.report.simulation_time = 0.0;
    self.report.frames = 0;

    report
  }

  /// Moves each system a motion carries to where it has its zone this frame, and its objects with it
  /// (`CCustomZone::OnMove`, `UpdateParent`), their sources taking on the zone's velocity.
  fn move_systems(level: &mut LevelSystems, motions: &mut LevelObjectMotions) {
    for system in &mut level.systems {
      let Some((transform, velocity)) = system.motion.as_deref().and_then(|name| motions.get_pose(name)) else {
        continue;
      };

      system.transform = transform;

      for object in system.objects.iter_mut() {
        object.update_parent(&transform, velocity);
      }
    }
  }

  /// Plays what each system's source plays: a planted system always; a zone's idle effect while it is enabled, stopped
  /// on Monolith while the camera stands past `FASTMODE_DISTANCE` (`o_switch_2_slow`) and played afresh once it comes
  /// back (`o_switch_2_fast`); a campfire's as `CZoneCampfire` switches them, following its campfire's state, its idle
  /// particles carried by the wind.
  fn place(level: &mut LevelSystems, eye: Vec3, now: u64, campfires: &mut LevelCampfires) {
    let wind: Vec3 = level.ambient.get_gust().get_velocity();
    let context: ParticleUpdateContext = ParticleUpdateContext {
      library: &level.library,
      rules: &level.rules,
      collider: level.collider.as_deref(),
    };
    // Only Monolith stops a slowed zone's idle particles.
    let is_slowed: bool = level.rules.get_engine() == XrayEngine::Extended;

    for system in &mut level.systems {
      let placement: (&Mat4, i32) = (&system.transform, system.seed);
      let objects: &mut PlacedObjects = &mut system.objects;
      let is_far: bool = is_slowed
        && system.zone_sphere.as_ref().is_some_and(|sphere| {
          sphere.get_distance(system.transform.w_axis.truncate().to_array(), eye.to_array()) > ZONE_FAST_DISTANCE
        });

      match &system.source {
        RenderParticleSource::Static { name } => objects.play(PlacedEffect::Idle, name, placement, &context, now),
        RenderParticleSource::Zone { .. } if is_far => objects.stop(PlacedEffect::Idle),
        RenderParticleSource::Zone { idle } => objects.play(PlacedEffect::Idle, idle, placement, &context, now),
        RenderParticleSource::Campfire {
          id,
          idle,
          disabled,
          enabling,
        } => {
          let campfire: Campfire = *campfires.get(*id);
          let at: u64 = campfires.get_now();

          match system.campfire_lit {
            // One placed out from the first plays its disabled effect once.
            None if !campfire.is_lit() => objects.play(PlacedEffect::Disabled, disabled, placement, &context, now),
            // A turn plays its effects (`GoEnabledState`, `GoDisabledState`); one that ended while nothing was placed,
            // the particles hidden, leaves only what the state it settled in keeps.
            Some(was_lit) if was_lit != campfire.is_lit() => {
              if campfire.is_lit() {
                objects.stop(PlacedEffect::Disabled);

                if campfire.is_turning() {
                  objects.play(PlacedEffect::Enabling, enabling, placement, &context, now);
                }
              } else if campfire.is_turning() {
                objects.play(PlacedEffect::Disabled, disabled, placement, &context, now);
              }
            }
            _ => {}
          }

          system.campfire_lit = Some(campfire.is_lit());

          // The zone's idle effect through the campfire's gates: played while lit and near, taking over from the
          // enabling one; stopped while out, or while far on Monolith.
          if campfire.is_lit() && !is_far {
            if campfire.can_play_idle(at) {
              objects.play(PlacedEffect::Idle, idle, placement, &context, now);
              objects.stop(PlacedEffect::Enabling);
            }
          } else if campfire.can_stop_idle(at) {
            objects.stop(PlacedEffect::Idle);
          }

          objects.carry(PlacedEffect::Idle, &system.transform, wind);
        }
      }
    }
  }

  /// Whether bounds in engine space stand within the frustum's planes in renderer space.
  fn is_in_view(planes: &[Vec4; 6], bounds: ParticleBounds) -> bool {
    let (center, radius) = bounds.get_sphere();
    let center: Vec3 = ParticleSprite::mirror(center);

    planes
      .iter()
      .all(|plane| plane.truncate().dot(center) + plane.w >= -radius)
  }

  /// Writes the frame's quads, growing the buffer past them, and the surfaces once after they are read.
  fn write_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    let size: u64 = (self.vertices.len() * size_of::<ParticleVertex>()) as u64;

    if size > self.vertex_buffer.size() {
      self.vertex_buffer = Self::create_storage(device, "particle vertices", size.next_power_of_two());
      self.buffers_generation += 1;
    }

    if !self.vertices.is_empty() {
      queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
    }

    if self.is_surfaces_dirty
      && let Some(level) = &self.systems
    {
      let records: Vec<ParticleSurfaceRecord> = level.surfaces.iter().map(|surface| surface.record).collect();

      if !records.is_empty() {
        queue.write_buffer(&self.surface_buffer, 0, bytemuck::cast_slice(&records));
      }

      self.is_surfaces_dirty = false;
    }
  }

  fn create_storage(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(label),
      size,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    })
  }
}

impl PlacedObjects {
  /// Plays an effect at the placement unless it already plays, as a zone makes its idle object only once.
  fn play(
    &mut self,
    effect: PlacedEffect,
    name: &str,
    (transform, seed): (&Mat4, i32),
    context: &ParticleUpdateContext,
    now: u64,
  ) {
    let slot: &mut Option<ParticleObject> = &mut self.0[effect.get_index()];

    if slot.is_some() {
      return;
    }

    // Each effect a sequence of its own, so a campfire's effects do not repeat one another.
    let Some(instance) = context.library.create(name, seed ^ ((effect.get_index() as i32) << 16)) else {
      return;
    };
    let mut object: ParticleObject = ParticleObject::new(instance);

    object.update_parent(transform, Vec3::ZERO);
    object.play(now, context);
    *slot = Some(object);
  }

  /// Moves an effect's sources at a velocity where it stands, as `CZoneCampfire::shedule_Update` carries its idle
  /// particles by the wind.
  fn carry(&mut self, effect: PlacedEffect, transform: &Mat4, velocity: Vec3) {
    if let Some(object) = &mut self.0[effect.get_index()] {
      object.update_parent(transform, velocity);
    }
  }

  /// Stops an effect at once, its particles gone with it: `Stop(FALSE)`, then `Destroy`.
  fn stop(&mut self, effect: PlacedEffect) {
    self.0[effect.get_index()] = None;
  }

  fn iter(&self) -> impl Iterator<Item = &ParticleObject> {
    self.0.iter().flatten()
  }

  fn iter_mut(&mut self) -> impl Iterator<Item = &mut ParticleObject> {
    self.0.iter_mut().flatten()
  }
}
