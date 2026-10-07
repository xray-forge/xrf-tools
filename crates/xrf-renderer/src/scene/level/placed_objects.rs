use glam::{Mat4, Vec3};
use xrf_particles::{ParticleObject, ParticleUpdateContext};

use crate::scene::level::placed_effect::PlacedEffect;

/// What plays at an emitter, an object a [`PlacedEffect`], each with whether its end was told to the world.
#[derive(Default)]
pub struct PlacedObjects([Option<(ParticleObject, bool)>; PlacedEffect::COUNT]);

impl PlacedObjects {
  /// Plays an effect where the emitter stands unless it already plays, as a zone makes its idle object only once; an
  /// effect the library lacks plays nothing.
  pub fn play(
    &mut self,
    effect: PlacedEffect,
    name: &str,
    (transform, seed): (&Mat4, i32),
    context: &ParticleUpdateContext,
    now: u64,
  ) {
    let slot: &mut Option<(ParticleObject, bool)> = &mut self.0[effect.get_index()];

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
    *slot = Some((object, false));
  }

  /// Stops an effect: deferred, it emits no more and what it emitted lives on (`Stop(TRUE)`); otherwise it goes at once
  /// with its particles (`Stop(FALSE)`, then `Destroy`).
  pub fn stop(&mut self, effect: PlacedEffect, is_deferred: bool) {
    let slot: &mut Option<(ParticleObject, bool)> = &mut self.0[effect.get_index()];

    match slot {
      Some((object, _)) if is_deferred => {
        if !object.is_stopping() {
          object.stop(true);
        }
      }
      _ => *slot = None,
    }
  }

  /// Moves every effect's sources where the emitter now stands, taking on its velocity (`UpdateParent`).
  pub fn move_to(&mut self, transform: &Mat4, velocity: Vec3) {
    for (object, _) in self.0.iter_mut().flatten() {
      object.update_parent(transform, velocity);
    }
  }

  /// Moves one effect's sources at a velocity where the emitter stands, as `CZoneCampfire::shedule_Update` carries its
  /// idle particles by the wind.
  pub fn carry(&mut self, effect: PlacedEffect, transform: &Mat4, velocity: Vec3) {
    if let Some((object, _)) = &mut self.0[effect.get_index()] {
      object.update_parent(transform, velocity);
    }
  }

  /// The effects that stopped playing since they were last asked for, each told once; they stay, so playing one again
  /// does nothing until it is stopped.
  pub fn take_finished(&mut self) -> Vec<PlacedEffect> {
    let mut finished: Vec<PlacedEffect> = Vec::new();

    for (index, slot) in self.0.iter_mut().enumerate() {
      if let Some((object, is_told)) = slot
        && !*is_told
        && !object.is_playing()
      {
        *is_told = true;
        finished.push(PlacedEffect::ALL[index]);
      }
    }

    finished
  }

  pub fn iter(&self) -> impl Iterator<Item = &ParticleObject> {
    self.0.iter().flatten().map(|(object, _)| object)
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ParticleObject> {
    self.0.iter_mut().flatten().map(|(object, _)| object)
  }
}
