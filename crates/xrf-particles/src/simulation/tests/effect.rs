use std::sync::Arc;

use glam::{Mat4, Vec3};

use crate::data::particle_effect::ParticleEffect;
use crate::data::particle_effect_flags::ParticleEffectFlags;
use crate::data::particle_effect_frame::ParticleEffectFrame;
use crate::simulation::particle_effect_instance::ParticleEffectInstance;
use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::tests::fixtures::{effect, kill_old, moving, source};

fn playing(definition: ParticleEffect) -> ParticleEffectInstance {
  let mut instance: ParticleEffectInstance = ParticleEffectInstance::new(Arc::new(definition), 1);

  instance.play();
  instance
}

#[test]
fn takes_whole_steps_and_no_more_than_three_at_once() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let mut instance: ParticleEffectInstance = playing(effect("one", 1, vec![source(1_000.0), moving()]));

  // 200 ms is six steps, clamped to three; the 2 ms left over carry.
  instance.update(200, &rules);
  assert_eq!(instance.get_pool().get_particles()[0].age, 0.033 + 0.033 + 0.033);

  instance.update(30, &rules);
  assert_eq!(instance.get_pool().get_particles()[0].age, 0.033 + 0.033 + 0.033);

  instance.update(1, &rules);
  assert_eq!(
    instance.get_pool().get_particles()[0].age,
    0.033 + 0.033 + 0.033 + 0.033
  );
}

#[test]
fn stops_itself_past_its_time_limit_and_plays_its_particles_out() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let mut definition: ParticleEffect = effect("timed", 10, vec![source(1_000.0), moving(), kill_old(0.1)]);

  definition.flags |= ParticleEffectFlags::TIME_LIMIT;
  definition.time_limit = Some(0.05);

  let mut instance: ParticleEffectInstance = playing(definition);

  // Two steps run (0.017 left, then below zero on the second), which stops the sources.
  instance.update(66, &rules);
  assert!(instance.is_playing());
  assert_eq!(instance.get_pool().len(), 10);

  // The particles age out at 0.1 s; once none is left the effect stops.
  for _ in 0..5 {
    instance.update(33, &rules);
  }

  assert!(instance.get_pool().is_empty());
  assert!(!instance.is_playing());
}

#[test]
fn drops_every_particle_on_a_stop_without_delay() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let mut instance: ParticleEffectInstance = playing(effect("cut", 4, vec![source(1_000.0)]));

  instance.update(33, &rules);
  instance.stop(false);

  assert!(instance.get_pool().is_empty());
  assert!(!instance.is_playing());
}

#[test]
fn advances_frames_only_when_framed_and_animated() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let mut definition: ParticleEffect = effect("frames", 1, vec![source(1_000.0)]);

  definition.flags |= ParticleEffectFlags::FRAMED | ParticleEffectFlags::ANIMATED;
  definition.frame = Some(ParticleEffectFrame {
    texture_size: (0.25, 0.25),
    reserved: (0.0, 0.0),
    frame_dimension_x: 4,
    frame_count: 16,
    frame_speed: 10.0,
  });

  let mut instance: ParticleEffectInstance = playing(definition);

  instance.update(33, &rules);

  // 10 frames a second for 0.033 s, stored times 255 and floored.
  assert_eq!(
    instance.get_pool().get_particles()[0].frame,
    (10.0 * 0.033 * 255.0f32).floor() as u16
  );
}

#[test]
fn places_its_actions_and_bounds_its_particles_grown_by_their_size() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let mut instance: ParticleEffectInstance = playing(effect("placed", 1, vec![source(1_000.0)]));

  instance.update_parent(&Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)), Vec3::ZERO);
  instance.update(33, &rules);

  assert_eq!(
    instance.get_pool().get_particles()[0].position,
    Vec3::new(10.0, 0.0, 0.0)
  );
  assert_eq!(instance.get_bounds().min, Vec3::new(9.0, -1.0, -1.0));
  assert_eq!(instance.get_bounds().max, Vec3::new(11.0, 1.0, 1.0));
}

#[test]
fn loops_without_a_time_limit() {
  let instance: ParticleEffectInstance = ParticleEffectInstance::new(Arc::new(effect("loop", 1, vec![])), 1);

  assert!(instance.is_looped());
  assert_eq!(instance.get_time_limit(), None);
}
