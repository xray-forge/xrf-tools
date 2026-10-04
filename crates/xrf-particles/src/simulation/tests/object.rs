use glam::{Mat4, Vec3};

use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_instance::ParticleInstance;
use crate::simulation::particle_library::ParticleLibrary;
use crate::simulation::particle_object::ParticleObject;
use crate::simulation::tests::fixtures::{context, effect, library, moving, source};

fn age_of(object: &ParticleObject) -> f32 {
  object
    .get_instance()
    .get_effects()
    .next()
    .map_or(0.0, |effect| effect.get_pool().get_particles()[0].age)
}

fn placed(library: &ParticleLibrary, rules: &ParticleEngineRules, now: u64) -> ParticleObject {
  let instance: ParticleInstance = library.create("smoke", 1).expect("effect");
  let mut object: ParticleObject = ParticleObject::new(instance);

  object.update_parent(&Mat4::from_translation(Vec3::new(400.0, 0.0, 0.0)), Vec3::ZERO);
  object.play(now, &context(library, rules));
  object
}

#[test]
fn takes_a_step_as_it_plays_and_loops_without_a_limit() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let library: ParticleLibrary = library(vec![effect("smoke", 1, vec![source(1_000.0), moving()])], vec![]);
  let object: ParticleObject = placed(&library, &rules, 1_000);

  assert!(object.is_looped());
  assert_eq!(age_of(&object), 0.033);
}

#[test]
fn updates_a_far_hidden_object_only_when_the_scheduler_comes_round() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let library: ParticleLibrary = library(vec![effect("smoke", 1, vec![source(1_000.0), moving()])], vec![]);
  let mut object: ParticleObject = placed(&library, &rules, 1_000);

  // Due at once, then 400 m away: the longest interval, 525 ms.
  object.advance(1_100, Vec3::ZERO, false, &context(&library, &rules));
  let after_schedule: f32 = age_of(&object);

  object.advance(1_400, Vec3::ZERO, false, &context(&library, &rules));
  assert_eq!(age_of(&object), after_schedule);

  // Drawn, it updates by everything since: 300 ms, clamped to three steps.
  object.advance(1_400, Vec3::ZERO, true, &context(&library, &rules));
  assert_eq!(age_of(&object), after_schedule + 0.033 + 0.033 + 0.033);
}
