use std::sync::Arc;

use glam::Vec3;

use crate::data::particle_effect::ParticleEffect;
use crate::data::particle_effect_collision::ParticleEffectCollision;
use crate::data::particle_effect_flags::ParticleEffectFlags;
use crate::simulation::particle_collider::ParticleCollider;
use crate::simulation::particle_contact::ParticleContact;
use crate::simulation::particle_effect_instance::ParticleEffectInstance;
use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_library::ParticleLibrary;
use crate::simulation::particle_update_context::ParticleUpdateContext;
use crate::simulation::tests::fixtures::{context, effect, moving, source_at};

/// A floor at height zero, met from above.
struct Floor;

impl ParticleCollider for Floor {
  fn pick(&self, origin: Vec3, direction: Vec3, range: f32, _: bool) -> Option<ParticleContact> {
    if origin.y <= 0.0 || direction.y >= 0.0 {
      return None;
    }

    let distance: f32 = origin.y / -direction.y;

    (distance <= range).then_some(ParticleContact {
      distance,
      normal: Vec3::Y,
    })
  }
}

/// One particle falling through the floor in its first step, colliding with flags.
fn falling(flags: u32) -> ParticleEffectInstance {
  let mut definition: ParticleEffect = effect(
    "sparks",
    1,
    vec![source_at(1_000.0, [0.0, 0.01, 0.0], [1.0, -1.0, 0.0]), moving()],
  );

  definition.flags |= flags;
  definition.collision = Some(ParticleEffectCollision {
    collide_one_minus_friction: 0.5,
    collide_resilience: 0.5,
    collide_sqr_cutoff: 0.0,
  });

  let mut instance: ParticleEffectInstance = ParticleEffectInstance::new(Arc::new(definition), 1);

  instance.play();
  instance
}

#[test]
fn reflects_a_particle_off_the_surface_it_crossed() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let library: ParticleLibrary = ParticleLibrary::default();
  let mut instance: ParticleEffectInstance = falling(ParticleEffectFlags::COLLISION);

  instance.update(
    33,
    &ParticleUpdateContext {
      library: &library,
      rules: &rules,
      collider: Some(&Floor),
    },
  );

  // Tangent (1, 0, 0) halved by friction, normal (0, -1, 0) reversed at half; moved from where it started.
  let particle = instance.get_pool().get_particles()[0];

  assert_eq!(particle.velocity, Vec3::new(0.5, 0.5, 0.0));
  assert_eq!(
    particle.position,
    Vec3::new(0.0, 0.01, 0.0) + Vec3::new(0.5, 0.5, 0.0) * 0.033
  );
}

#[test]
fn removes_a_particle_on_contact_when_the_effect_asks() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let library: ParticleLibrary = ParticleLibrary::default();
  let mut instance: ParticleEffectInstance =
    falling(ParticleEffectFlags::COLLISION | ParticleEffectFlags::COLLISION_DELETE);

  instance.update(
    33,
    &ParticleUpdateContext {
      library: &library,
      rules: &rules,
      collider: Some(&Floor),
    },
  );

  assert!(instance.get_pool().is_empty());
}

#[test]
fn passes_through_without_a_collider() {
  let rules: ParticleEngineRules = ParticleEngineRules::default();
  let library: ParticleLibrary = ParticleLibrary::default();
  let mut instance: ParticleEffectInstance = falling(ParticleEffectFlags::COLLISION);

  instance.update(33, &context(&library, &rules));

  assert!(instance.get_pool().get_particles()[0].position.y < 0.0);
}
