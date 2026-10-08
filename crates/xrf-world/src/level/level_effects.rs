use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec3};
use xrf_engine_target::XrayEngine;
use xrf_particles::ParticleLibrary;
use xrf_renderer::{
  AmbientGust, LoaderReceiver, ParticleEmitterProxy, PlacedEffect, RenderLevelParticles, RenderLevelSource,
  RenderParticleDefinitions, RenderParticleSource, RenderSceneUpdate, RenderViewOptions, RenderWorkers,
  ZONE_FAST_DISTANCE, take_answer,
};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

use crate::contract::world_ambient_report::WorldAmbientReport;
use crate::contract::world_toggles::WorldToggles;
use crate::level::ambient_frame::AmbientFrame;
use crate::level::camera_hemi::CameraHemi;
use crate::level::campfire::Campfire;
use crate::level::effects_frame::EffectsFrame;
use crate::level::level_ambient_effects::{AmbientSystems, LevelAmbientEffects};
use crate::level::level_campfires::LevelCampfires;
use crate::level::level_object_motions::LevelObjectMotions;
use crate::level::placed_emitter::PlacedEmitter;

/// A level's effects as the world plays them: its particle systems, read on a loader thread and each placed at an
/// emitter of the scene's; the weather's ambient effects and the wind they bring; its campfires' switching; and the
/// object motions its moving zones follow. Its posts tell the scene what plays where; the scene simulates and draws it,
/// and tells which effects finished.
pub struct LevelEffects {
  pending: Option<LoaderReceiver<Result<Option<RenderLevelParticles>, String>>>,
  systems: Option<EffectSystems>,
  emitters: ProxyAllocator<ParticleEmitterProxy>,
  /// The emitters whose effects finished and that the ambient effects have not yet been told of.
  finished: Vec<ProxyHandle<ParticleEmitterProxy>>,
  ambient: LevelAmbientEffects,
  campfires: LevelCampfires,
  motions: LevelObjectMotions,
  workers: RenderWorkers,
  started: Instant,
}

/// What the read particles keep: what the ambient effects check their effects against, how lit the camera stands, and
/// every placement.
struct EffectSystems {
  library: Arc<ParticleLibrary>,
  engine: XrayEngine,
  hemi: CameraHemi,
  placed: Vec<PlacedEmitter>,
}

impl LevelEffects {
  pub fn start(source: &Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();
    let reader: Arc<dyn RenderLevelSource> = Arc::clone(source);

    workers.spawn(move || {
      let particles = reader.read_particles().map_err(|error| error.to_string());

      if let Err(error) = &particles {
        log::error!("The level's particles cannot be played: {error}");
      }

      let _ = sender.send(particles);
    });

    Self {
      pending: Some(receiver),
      systems: None,
      emitters: ProxyAllocator::new(),
      finished: Vec::new(),
      ambient: LevelAmbientEffects::default(),
      campfires: LevelCampfires::new(),
      motions: LevelObjectMotions::new(source, workers),
      workers: workers.clone(),
      started: Instant::now(),
    }
  }

  /// Whether its loader has answered, whatever it answered.
  pub fn is_read(&self) -> bool {
    self.pending.is_none()
  }

  /// Plays an ambient effect on the next frame, ending the one playing, without waiting.
  pub fn play_ambient_now(&mut self) {
    self.ambient.play_now();
  }

  /// Where the ambient effects stand, none until the particles are read.
  pub fn report_ambient(&self) -> Option<WorldAmbientReport> {
    self
      .systems
      .as_ref()
      .map(|systems| self.ambient.report(self.get_now(), systems.hemi.is_indoors()))
  }

  /// Takes the effects the scene finished, which the ambient effects are told of on their next update.
  pub fn note_finished(&mut self, finished: Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)>) {
    self.finished.extend(finished.into_iter().map(|(handle, _)| handle));
  }

  /// Moves the effects on a frame and posts what the scene plays: takes the read particles, plays the weather's ambient
  /// effects, switches the campfires and moves the object motions, then plays what each placement plays. `eye` is where
  /// the camera stands in engine space; `lights` the campfires and motions the level's lights follow. Frozen while
  /// particles are not drawn, and played as though indoors while the ambient effects are switched off.
  pub fn advance(
    &mut self,
    updates: &mut Vec<RenderSceneUpdate>,
    (options, toggles, ambient): (&RenderViewOptions, &WorldToggles, Option<AmbientFrame<'_>>),
    eye: Vec3,
    lights: (&[u16], &[String]),
  ) -> EffectsFrame {
    let now: u64 = self.get_now();
    let is_playing: bool = options.show.is_particled && options.mode.is_lit;

    self.take_read(updates);

    if is_playing && let Some(systems) = &mut self.systems {
      systems.hemi.advance(eye, now);
      self.ambient.update(
        updates,
        AmbientSystems {
          library: &systems.library,
          engine: systems.engine,
          emitters: &mut self.emitters,
          finished: &std::mem::take(&mut self.finished),
        },
        ambient,
        (eye, systems.hemi.is_indoors() || !toggles.is_ambient_played),
        now,
      );
    }

    // The campfires switch and the moving zones move whether or not their particles are drawn, so their lights follow
    // them alike.
    self.campfires.prepare(toggles.is_campfire_lit);
    self.motions.prepare();

    let placed: &[PlacedEmitter] = self.systems.as_ref().map_or(&[], |systems| &systems.placed);
    let mut campfire_shares: HashMap<u16, f32> = HashMap::new();
    let mut motions: HashMap<String, (Mat4, Vec3)> = HashMap::new();

    for id in placed
      .iter()
      .filter_map(|emitter| match *emitter.source {
        RenderParticleSource::Campfire { id, .. } => Some(id),
        _ => None,
      })
      .chain(lights.0.iter().copied())
    {
      campfire_shares.insert(id, self.campfires.get_light_share(id));
    }

    for name in placed
      .iter()
      .filter_map(|emitter| emitter.motion.as_deref())
      .chain(lights.1.iter().map(String::as_str))
    {
      if !motions.contains_key(name)
        && let Some(pose) = self.motions.get_pose(name)
      {
        motions.insert(name.to_owned(), pose);
      }
    }

    let gust: AmbientGust = self.ambient.get_gust();

    if is_playing && let Some(systems) = &mut self.systems {
      Self::place(
        updates,
        systems,
        (eye, gust.get_velocity()),
        (&mut self.campfires, &motions),
      );
    }

    EffectsFrame {
      gust,
      campfire_shares,
      motions,
      camera_hemi: self.systems.as_ref().and_then(|systems| systems.hemi.get_smooth()),
    }
  }

  /// The effects' clock, in milliseconds since the level began opening.
  fn get_now(&self) -> u64 {
    self.started.elapsed().as_millis() as u64
  }

  /// Takes the particles once their loader read them: posts what they are made from and an emitter for each placement.
  fn take_read(&mut self, updates: &mut Vec<RenderSceneUpdate>) {
    let Some(read) = take_answer(&mut self.pending, "particles") else {
      return;
    };
    let Ok(Some(read)) = read else {
      return;
    };
    let RenderLevelParticles {
      library,
      rules,
      placements,
      surfaces,
      collider,
      hemi,
    } = read;

    updates.push(RenderSceneUpdate::AddParticles(RenderParticleDefinitions {
      library: Arc::clone(&library),
      rules,
      surfaces,
      collider,
    }));

    let placed: Vec<PlacedEmitter> = placements
      .into_iter()
      .enumerate()
      .map(|(index, placement)| {
        let handle: ProxyHandle<ParticleEmitterProxy> = self.emitters.allocate();
        let transform: Mat4 = Mat4::from_cols_array(&placement.transform);

        updates.push(RenderSceneUpdate::AddEmitter {
          handle,
          transform,
          seed: index as i32 + 1,
        });

        PlacedEmitter::new(
          handle,
          placement.source,
          transform,
          (placement.motion, placement.zone_sphere),
        )
      })
      .collect();

    log::info!("Level particles {} placed systems", placed.len());

    self.systems = Some(EffectSystems {
      library,
      engine: rules.get_engine(),
      hemi: CameraHemi::new(hemi, &self.workers),
      placed,
    });
  }

  /// Plays what each placement's source plays: a planted system always; a zone's idle effect while it is enabled,
  /// stopped on Monolith while the camera stands past `FASTMODE_DISTANCE` (`o_switch_2_slow`) and played afresh once it
  /// comes back (`o_switch_2_fast`); a campfire's as `CZoneCampfire` switches them, following its campfire's state, its
  /// idle particles carried by the wind. A placement a motion carries moves first.
  fn place(
    updates: &mut Vec<RenderSceneUpdate>,
    systems: &mut EffectSystems,
    (eye, wind): (Vec3, Vec3),
    (campfires, motions): (&mut LevelCampfires, &HashMap<String, (Mat4, Vec3)>),
  ) {
    // Only Monolith stops a slowed zone's idle particles.
    let is_slowed: bool = systems.engine == XrayEngine::Extended;

    for emitter in &mut systems.placed {
      if let Some(&(transform, velocity)) = emitter.motion.as_ref().and_then(|name| motions.get(name)) {
        emitter.move_to(updates, transform, velocity);
      }

      let is_far: bool = is_slowed
        && emitter.zone_sphere.as_ref().is_some_and(|sphere| {
          sphere.get_distance(emitter.transform.w_axis.truncate().to_array(), eye.to_array()) > ZONE_FAST_DISTANCE
        });

      let source: Arc<RenderParticleSource> = Arc::clone(&emitter.source);

      match &*source {
        RenderParticleSource::Static { name } => emitter.play(updates, PlacedEffect::Idle, name),
        RenderParticleSource::Zone { .. } if is_far => emitter.stop(updates, PlacedEffect::Idle),
        RenderParticleSource::Zone { idle } => emitter.play(updates, PlacedEffect::Idle, idle),
        RenderParticleSource::Campfire {
          id,
          idle,
          disabled,
          enabling,
        } => {
          let campfire: Campfire = *campfires.get(*id);
          let at: u64 = campfires.get_now();

          match emitter.campfire_lit {
            // One placed out from the first plays its disabled effect once.
            None if !campfire.is_lit() => emitter.play(updates, PlacedEffect::Disabled, disabled),
            // A turn plays its effects (`GoEnabledState`, `GoDisabledState`); one that ended while nothing was placed,
            // the particles hidden, leaves only what the state it settled in keeps.
            Some(was_lit) if was_lit != campfire.is_lit() => {
              if campfire.is_lit() {
                emitter.stop(updates, PlacedEffect::Disabled);

                if campfire.is_turning() {
                  emitter.play(updates, PlacedEffect::Enabling, enabling);
                }
              } else if campfire.is_turning() {
                emitter.play(updates, PlacedEffect::Disabled, disabled);
              }
            }
            _ => {}
          }

          emitter.campfire_lit = Some(campfire.is_lit());

          // The zone's idle effect through the campfire's gates: played while lit and near, taking over from the
          // enabling one; stopped while out, or while far on Monolith.
          if campfire.is_lit() && !is_far {
            if campfire.can_play_idle(at) {
              emitter.play(updates, PlacedEffect::Idle, idle);
              emitter.stop(updates, PlacedEffect::Enabling);
            }
          } else if campfire.can_stop_idle(at) {
            emitter.stop(updates, PlacedEffect::Idle);
          }

          emitter.carry(updates, PlacedEffect::Idle, wind);
        }
      }
    }
  }
}
