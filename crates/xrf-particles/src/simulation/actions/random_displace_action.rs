use glam::Mat4;

use crate::data::actions::particle_action_random_displace::ParticleActionRandomDisplace;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PARandomDisplace`: moves each particle by a random displacement from a volume, scaled to the step.
pub(crate) struct RandomDisplaceAction {
  displacement_local: ParticleVolume,
  displacement: ParticleVolume,
}

impl RandomDisplaceAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self
      .displacement
      .transform_direction_from(&self.displacement_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let (particles, random) = pool.split_mut();

    for m in particles {
      m.position += self.displacement.generate(random) * step.dt;
    }
  }
}

impl From<&ParticleActionRandomDisplace> for RandomDisplaceAction {
  fn from(action: &ParticleActionRandomDisplace) -> Self {
    let displacement: ParticleVolume = ParticleVolume::from(&action.gen_disp);

    Self {
      displacement_local: displacement.clone(),
      displacement,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::RandomDisplaceAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn moves_by_a_point_volume_times_the_step() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let point: ParticleVolume = ParticleVolume {
      kind: ParticleVolume::POINT,
      p1: Vec3::new(0.0, 4.0, 0.0),
      p2: Vec3::ZERO,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    };
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    RandomDisplaceAction {
      displacement_local: point.clone(),
      displacement: point,
    }
    .execute(&mut pool, &ParticleActionStep::new(0.25, &rules));

    assert_eq!(pool.get_particles()[0].position, Vec3::new(0.0, 1.0, 0.0));
  }
}
