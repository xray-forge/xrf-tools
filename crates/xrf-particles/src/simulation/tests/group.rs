use std::sync::Arc;

use crate::data::particle_effect::ParticleEffect;
use crate::data::particle_effect_flags::ParticleEffectFlags;
use crate::data::particle_group_effect::ParticleGroupEffect;
use crate::data::particle_group_effect_flags::ParticleGroupEffectFlags;
use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_group_instance::ParticleGroupInstance;
use crate::simulation::particle_library::ParticleLibrary;
use crate::simulation::tests::fixtures::{effect, group, group_effect, kill_old, library, moving, source};

fn playing(library: &ParticleLibrary, name: &str) -> ParticleGroupInstance {
  let mut instance: ParticleGroupInstance =
    ParticleGroupInstance::new(Arc::clone(library.get_group(name).expect("group")), library, 1);

  instance.play();
  instance
}

fn timed_child(name: &str) -> ParticleEffect {
  let mut child: ParticleEffect = effect(name, 1, vec![source(1_000.0), moving()]);

  child.flags |= ParticleEffectFlags::TIME_LIMIT;
  child.time_limit = Some(1.0);
  child
}

#[test]
fn plays_an_effect_as_the_clock_passes_its_start_and_stops_it_at_its_end() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let library: ParticleLibrary = library(
    vec![effect("steam", 4, vec![source(1_000.0)])],
    vec![group(
      "vent",
      0.0,
      vec![group_effect("steam", 0.05, 0.1, ParticleGroupEffectFlags::ENABLED)],
    )],
  );
  let mut instance: ParticleGroupInstance = playing(&library, "vent");

  instance.update(33, &library, &rules);
  assert_eq!(instance.get_effects().next().map(|it| it.is_playing()), Some(false));

  instance.update(33, &library, &rules);
  assert_eq!(instance.get_effects().next().map(|it| it.is_playing()), Some(true));

  // Past 0.1 s the effect stops at once, having no deferred stop, and so does the group timed to it.
  instance.update(33, &library, &rules);
  instance.update(33, &library, &rules);
  assert_eq!(instance.get_effects().next().map(|it| it.is_playing()), Some(false));
  assert_eq!(instance.get_time_limit(), 0.1);
  assert!(!instance.is_playing());
}

#[test]
fn never_plays_a_disabled_effect() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let library: ParticleLibrary = library(
    vec![effect("steam", 4, vec![source(1_000.0)])],
    vec![group("vent", 0.0, vec![group_effect("steam", 0.0, 0.0, 0)])],
  );
  let mut instance: ParticleGroupInstance = playing(&library, "vent");

  instance.update(33, &library, &rules);

  assert_eq!(instance.get_effects().next().map(|it| it.is_playing()), Some(false));
}

#[test]
fn follows_each_particle_with_a_child_and_frees_it_when_the_particle_dies() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let mut parent: ParticleGroupEffect = group_effect(
    "sparks",
    0.0,
    0.0,
    ParticleGroupEffectFlags::ENABLED | ParticleGroupEffectFlags::ON_PLAY_CHILD,
  );
  let mut sparks: ParticleEffect = effect("sparks", 2, vec![source(1_000.0), moving(), kill_old(0.05)]);

  parent.on_play_child_name = String::from("trail");
  sparks.flags |= ParticleEffectFlags::TIME_LIMIT;
  sparks.time_limit = Some(0.04);

  let library: ParticleLibrary = library(
    vec![
      sparks,
      effect("trail", 1, vec![source(1_000.0), moving(), kill_old(0.1)]),
    ],
    vec![group("fire", 0.0, vec![parent])],
  );
  let mut instance: ParticleGroupInstance = playing(&library, "fire");

  // Two sparks born and moved up 0.033 m, each followed by a trail placed there, which emits there.
  instance.update(33, &library, &rules);

  let effects: Vec<_> = instance.get_effects().collect();

  assert_eq!(effects.len(), 3);
  assert_eq!(effects[1].get_pool().len(), 1);
  assert_eq!(effects[1].get_pool().get_particles()[0].position.y, 0.033 + 0.033);

  // The sparks stop, then die past 0.05 s; each trail stops with its spark and plays its particle out.
  instance.update(33, &library, &rules);
  instance.update(33, &library, &rules);

  let effects: Vec<_> = instance.get_effects().collect();

  assert_eq!(effects.len(), 3);
  assert!(effects[0].get_pool().is_empty());
  assert!(effects[1].is_playing());

  // The trails' particles age out past 0.1 s, and the finished trails are dropped.
  instance.update(33, &library, &rules);

  assert_eq!(instance.get_effects().count(), 1);
}

#[test]
fn starts_a_free_child_on_every_birth() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let mut parent: ParticleGroupEffect = group_effect(
    "sparks",
    0.0,
    0.0,
    ParticleGroupEffectFlags::ENABLED | ParticleGroupEffectFlags::ON_BIRTH_CHILD,
  );

  parent.on_birth_child_name = String::from("puff");

  let library: ParticleLibrary = library(
    vec![effect("sparks", 3, vec![source(1_000.0)]), timed_child("puff")],
    vec![group("fire", 0.0, vec![parent])],
  );
  let mut instance: ParticleGroupInstance = playing(&library, "fire");

  instance.update(33, &library, &rules);

  assert_eq!(
    instance
      .get_effects()
      .filter(|it| it.get_definition().name == "puff")
      .count(),
    3
  );
}
