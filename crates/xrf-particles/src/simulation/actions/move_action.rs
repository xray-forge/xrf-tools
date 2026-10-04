use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAMove`: ages every particle and moves it by its velocity, remembering where it was.
pub(crate) struct MoveAction;

impl MoveAction {
  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    for m in pool.get_particles_mut() {
      m.age += step.dt;
      m.previous_position = m.position;
      m.position += m.velocity * step.dt;
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::MoveAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn moves_and_ages_by_the_step() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::X,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::new(0.0, 2.0, 0.0),
      Vec4::ONE,
      1.0,
    );
    MoveAction.execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    let particle = pool.get_particles()[0];

    assert_eq!(particle.position, Vec3::new(1.0, 1.0, 0.0));
    assert_eq!(particle.previous_position, Vec3::X);
    assert_eq!(particle.age, 1.5);
  }
}
