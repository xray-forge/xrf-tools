use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec3, Vec4};
use rayon::prelude::*;
use wgpu::util::DeviceExt;
use xrf_material::{XraySurfaceDraw, XraySurfaceSampler};
use xrf_particles::{
  ParticleBounds, ParticleCollider, ParticleEffectInstance, ParticleEngineRules, ParticleLibrary, ParticleObject,
  ParticleUpdateContext,
};
use xrf_renderer_core::{ProxyHandle, ProxyStore};

use crate::camera::camera_view::CameraView;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_particle_definitions::RenderParticleDefinitions;
use crate::pass::particle_blend::ParticleBlend;
use crate::pass::particle_surface_record::ParticleSurfaceRecord;
use crate::scene::level::particle_emitter_proxy::ParticleEmitterProxy;
use crate::scene::level::particle_sprite::ParticleSprite;
use crate::scene::level::particles_view::ParticlesView;
use crate::scene::level::placed_effect::PlacedEffect;
use crate::scene::level::placed_objects::PlacedObjects;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;
use crate::thread::frame_task::FrameTask;
use crate::thread::render_workers::RenderWorkers;

/// The sampler a distorting effect's `l_special` pass binds its distortion map to.
const DISTORTION_SAMPLER: &str = "s_distort";

/// A level's particles: the effects and groups the world read and the emitters it placed, each frame stepped on the
/// workers as the engine schedules them while the frame is prepared, and the effects each view sees filled into its
/// quads far to near for the particle pass (`ParticlesView`), which draws their colour and then their distortion. The
/// world says what plays where; this simulates it, and each view draws it.
pub struct LevelParticles {
  systems: Option<LevelSystems>,
  /// Empty while a step holds them: whatever reads or changes them settles the step first.
  emitters: ProxyStore<ParticleEmitterProxy>,
  /// The step offered to the workers, which answers the emitters it took.
  step: Option<FrameTask<ParticleStep>>,
  /// The effects that stopped playing this frame, told back to the world.
  finished: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
  workers: RenderWorkers,
  started: Instant,
  /// Each surface's record, which every view's particle pass reads.
  surface_buffer: wgpu::Buffer,
  report: ParticlesTally,
}

/// What the effects are made from, and how each draws.
struct LevelSystems {
  library: Arc<ParticleLibrary>,
  rules: ParticleEngineRules,
  collider: Option<Arc<dyn ParticleCollider>>,
  surfaces: Vec<ParticleSurface>,
  /// Each effect definition's surface, by the definition's address.
  surface_of: HashMap<usize, u32>,
  texture_slots: Vec<u32>,
}

/// How an effect's sprite draws: its equation, none for one drawing no colour, whether it distorts, and its record.
struct ParticleSurface {
  blend: Option<ParticleBlend>,
  is_distorting: bool,
  record: ParticleSurfaceRecord,
}

/// A step's outcome: the emitters it stepped, the effects that stopped, what they came to and what it took.
struct ParticleStep {
  emitters: ProxyStore<ParticleEmitterProxy>,
  finished: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>,
  report: RenderParticlesReport,
  time: f32,
}

/// What the frames since the last report came to.
#[derive(Default)]
struct ParticlesTally {
  last: RenderParticlesReport,
  simulation_time: f32,
  frames: u32,
}

impl LevelParticles {
  /// Objects a worker steps at least: fewer would cost more waking it than it saves.
  const STEPPED_PER_WORKER: usize = 8;

  pub fn new(device: &wgpu::Device, workers: &RenderWorkers) -> Self {
    Self {
      systems: None,
      emitters: ProxyStore::new(),
      step: None,
      finished: Vec::new(),
      workers: workers.clone(),
      started: Instant::now(),
      surface_buffer: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("particle surfaces"),
        size: size_of::<ParticleSurfaceRecord>() as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
      }),
      report: ParticlesTally::default(),
    }
  }

  /// Takes what the effects are made from, asking for every sprite's textures.
  pub fn add_definitions(
    &mut self,
    device: &wgpu::Device,
    (textures, source): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
    read: RenderParticleDefinitions,
  ) {
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

    log::info!("Native viewport particles {} effect surfaces", surfaces.len());

    let mut records: Vec<ParticleSurfaceRecord> = surfaces.iter().map(|surface| surface.record).collect();

    // A binding cannot be empty: no surface is bound as one.
    if records.is_empty() {
      records.push(ParticleSurfaceRecord::default());
    }

    self.surface_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("particle surfaces"),
      contents: bytemuck::cast_slice(&records),
      usage: wgpu::BufferUsages::STORAGE,
    });
    self.systems = Some(LevelSystems {
      library: read.library,
      rules: read.rules,
      collider: read.collider,
      surfaces,
      surface_of,
      texture_slots,
    });
  }

  /// Places an emitter at the handle the world allocated, nothing playing at it yet.
  pub fn add_emitter(&mut self, handle: ProxyHandle<ParticleEmitterProxy>, transform: Mat4, seed: i32) {
    self.settle();
    self.emitters.insert(
      handle,
      ParticleEmitterProxy {
        transform,
        seed,
        objects: PlacedObjects::default(),
      },
    );
  }

  /// Takes an emitter out with whatever plays at it.
  pub fn remove_emitter(&mut self, handle: ProxyHandle<ParticleEmitterProxy>) {
    self.settle();
    self.emitters.remove(handle);
  }

  /// Plays an effect at an emitter unless it already plays there; nothing before the definitions are in.
  pub fn play(&mut self, handle: ProxyHandle<ParticleEmitterProxy>, effect: PlacedEffect, name: &str) {
    self.settle();

    let now: u64 = self.started.elapsed().as_millis() as u64;
    let (Some(systems), Some(emitter)) = (&self.systems, self.emitters.get_mut(handle)) else {
      return;
    };
    let context: ParticleUpdateContext = ParticleUpdateContext {
      library: &systems.library,
      rules: &systems.rules,
      collider: systems.collider.as_deref(),
    };

    emitter
      .objects
      .play(effect, name, (&emitter.transform, emitter.seed), &context, now);
  }

  /// Stops an effect playing at an emitter: at once, or letting its particles live out their lives (`is_deferred`).
  pub fn stop(&mut self, handle: ProxyHandle<ParticleEmitterProxy>, effect: PlacedEffect, is_deferred: bool) {
    self.settle();

    if let Some(emitter) = self.emitters.get_mut(handle) {
      emitter.objects.stop(effect, is_deferred);
    }
  }

  /// Moves an emitter and every effect playing at it, their sources taking on its velocity.
  pub fn move_emitter(&mut self, handle: ProxyHandle<ParticleEmitterProxy>, transform: Mat4, velocity: Vec3) {
    self.settle();

    if let Some(emitter) = self.emitters.get_mut(handle) {
      emitter.transform = transform;
      emitter.objects.move_to(&transform, velocity);
    }
  }

  /// Moves one effect's sources at a velocity where its emitter stands.
  pub fn carry(&mut self, handle: ProxyHandle<ParticleEmitterProxy>, effect: PlacedEffect, velocity: Vec3) {
    self.settle();

    if let Some(emitter) = self.emitters.get_mut(handle) {
      emitter.objects.carry(effect, &emitter.transform, velocity);
    }
  }

  /// The effects that stopped playing since the last call, for the world.
  pub fn take_finished(&mut self) -> Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)> {
    self.settle();

    std::mem::take(&mut self.finished)
  }

  /// Starts stepping every effect playing as the engine schedules them on the workers, as `view` sees them (the scene's
  /// first view, which stands for the actor); nothing while particles are not drawn.
  pub fn start_simulation(&mut self, view: &CameraView, options: &RenderViewOptions) {
    self.settle();

    let Some(level) = self.systems.as_ref() else {
      return;
    };

    if !options.show.is_particled || !options.mode.is_lit {
      self.report.last = RenderParticlesReport::default();

      return;
    }

    let now: u64 = self.started.elapsed().as_millis() as u64;
    let eye: Vec3 = ParticleSprite::mirror(view.position);
    let planes: [Vec4; 6] = view.get_planes();
    let (library, rules, collider) = (Arc::clone(&level.library), level.rules, level.collider.clone());
    let mut emitters: ProxyStore<ParticleEmitterProxy> = std::mem::take(&mut self.emitters);

    self.step = Some(self.workers.spawn_frame_task(move |is_parallel: bool| {
      profiling::scope!("particles");

      let started: Instant = Instant::now();
      let context: ParticleUpdateContext = ParticleUpdateContext {
        library: &library,
        rules: &rules,
        collider: collider.as_deref(),
      };
      // Most objects are neither in view nor due for the scheduler: only the rest are stepped, a few to a worker.
      let mut due: Vec<(&mut ParticleObject, bool)> = emitters
        .as_mut_slice()
        .iter_mut()
        .flat_map(|emitter| emitter.objects.iter_mut())
        .filter_map(|object| {
          let is_in_view: bool = Self::is_in_view(&planes, object.get_instance().get_bounds());

          object.is_due(now, is_in_view).then_some((object, is_in_view))
        })
        .collect();
      let advance = |(object, is_in_view): &mut (&mut ParticleObject, bool)| -> u32 {
        u32::from(object.advance(now, eye, *is_in_view, &context))
      };
      // A step the frame took back for want of a free worker runs on the render thread, outside the pool.
      let simulated: u32 = if is_parallel {
        due
          .par_iter_mut()
          .with_min_len(Self::STEPPED_PER_WORKER)
          .map(advance)
          .sum()
      } else {
        due.iter_mut().map(advance).sum()
      };
      let mut finished: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)> = Vec::new();
      let mut report: RenderParticlesReport = RenderParticlesReport {
        simulated,
        ..RenderParticlesReport::default()
      };

      for (handle, emitter) in emitters.iter_mut() {
        finished.extend(
          emitter
            .objects
            .take_finished()
            .into_iter()
            .map(|effect| (handle, effect)),
        );

        for effect in emitter
          .objects
          .iter()
          .flat_map(|object| object.get_instance().get_effects())
        {
          report.effects += u32::from(effect.is_playing());
          report.particles += effect.get_pool().len() as u32;
        }
      }

      ParticleStep {
        emitters,
        finished,
        report,
        time: started.elapsed().as_secs_f32() * 1000.0,
      }
    }));
  }

  /// Joins the step offered to the workers, if one was, and takes back its emitters and what stopped.
  fn settle(&mut self) {
    let Some(step) = self.step.take() else {
      return;
    };

    profiling::scope!("particles wait");

    let Some(ParticleStep {
      emitters,
      finished,
      report,
      time,
    }) = step.join()
    else {
      log::error!("A particle step ended without answering; its effects are gone");

      return;
    };

    self.emitters = emitters;
    self.finished.extend(finished);
    self.report.last = report;
    self.report.simulation_time += time;
    self.report.frames += 1;
  }

  /// Fills a view's quads with the effects in its camera, far to near, once the frame's step is in; none while
  /// particles are not drawn in it.
  pub fn fill(&mut self, view: &CameraView, options: &RenderViewOptions, into: &mut ParticlesView) {
    into.clear();
    self.settle();

    let Some(level) = self.systems.as_ref() else {
      return;
    };

    if !options.show.is_particled || !options.mode.is_lit {
      return;
    }

    let eye: Vec3 = ParticleSprite::mirror(view.position);
    let planes: [Vec4; 6] = view.get_planes();
    let mut drawn: Vec<(f32, &ParticleEffectInstance, u32, &ParticleSurface)> = Vec::new();

    for effect in self
      .emitters
      .as_slice()
      .iter()
      .flat_map(|emitter| emitter.objects.iter())
      .flat_map(|object| object.get_instance().get_effects())
    {
      let Some(&surface) = level
        .surface_of
        .get(&(std::ptr::from_ref(effect.get_definition()) as usize))
      else {
        continue;
      };
      let drawing: &ParticleSurface = &level.surfaces[surface as usize];
      let is_drawing: bool = drawing.blend.is_some() || drawing.is_distorting;

      if is_drawing && !effect.get_pool().is_empty() && Self::is_in_view(&planes, effect.get_bounds()) {
        let (center, _) = effect.get_bounds().get_sphere();

        drawn.push((center.distance_squared(eye), effect, surface, drawing));
      }
    }

    // Far to near, as `mapSorted` draws what blends and `mapDistort` what distorts; every effect is sorted, as nothing
    // else orders them.
    drawn.sort_by(|a, b| b.0.total_cmp(&a.0));

    let sprite: ParticleSprite = ParticleSprite::new(view.view);

    for (_, effect, surface, drawing) in drawn {
      into.push_effect((&sprite, effect), surface, (drawing.blend, drawing.is_distorting));
    }
  }

  /// Every surface's record, which each view's particle pass reads, and how many times it was made.
  pub fn get_surfaces(&self) -> &wgpu::Buffer {
    &self.surface_buffer
  }

  /// The texture slots the sprites sample.
  pub fn get_texture_slots(&self) -> &[u32] {
    self
      .systems
      .as_ref()
      .map_or(&[], |systems| systems.texture_slots.as_slice())
  }

  /// What the last frame's simulation came to, with its mean cost since the last report; `drawn` is what a view drew
  /// of it.
  pub fn take_report(&mut self, drawn: u32) -> RenderParticlesReport {
    self.settle();

    let report: RenderParticlesReport = RenderParticlesReport {
      simulation_time: self.report.simulation_time / self.report.frames.max(1) as f32,
      drawn,
      ..self.report.last
    };

    self.report.simulation_time = 0.0;
    self.report.frames = 0;

    report
  }

  /// Whether bounds in engine space stand within the frustum's planes in renderer space.
  fn is_in_view(planes: &[Vec4; 6], bounds: ParticleBounds) -> bool {
    let (center, radius) = bounds.get_sphere();
    let center: Vec3 = ParticleSprite::mirror(center);

    planes
      .iter()
      .all(|plane| plane.truncate().dot(center) + plane.w >= -radius)
  }
}
