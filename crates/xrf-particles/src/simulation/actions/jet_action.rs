use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_jet::ParticleActionJet;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PAJet`: accelerates particles near a centre by a random acceleration from a volume, falling off with distance.
pub(crate) struct JetAction {
  center_local: Vec3,
  acceleration_local: ParticleVolume,
  center: Vec3,
  acceleration: ParticleVolume,
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl JetAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.center = matrix.transform_point3(self.center_local);
    self
      .acceleration
      .transform_direction_from(&self.acceleration_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;
    let (particles, random) = pool.split_mut();

    for m in particles {
      let distance_sqr: f32 = (m.position - self.center).length_squared();

      if !is_limited || distance_sqr < max_radius_sqr {
        let acceleration: Vec3 = self.acceleration.generate(random);

        m.velocity += acceleration * (magdt / (distance_sqr + self.epsilon));
      }
    }
  }
}

impl From<&ParticleActionJet> for JetAction {
  fn from(action: &ParticleActionJet) -> Self {
    let center: Vec3 = Vec3::from_array(action.center.to_array());
    let acceleration: ParticleVolume = ParticleVolume::from(&action.acc);

    Self {
      center_local: center,
      acceleration_local: acceleration.clone(),
      center,
      acceleration,
      magnitude: action.magnitude,
      epsilon: action.epsilon,
      max_radius: action.max_radius,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::JetAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn accelerates_inside_the_radius_by_the_inverse_square_distance() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let point: ParticleVolume = ParticleVolume {
      kind: ParticleVolume::POINT,
      p1: Vec3::Y,
      p2: Vec3::ZERO,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    };
    let action: JetAction = JetAction {
      center_local: Vec3::ZERO,
      acceleration_local: point.clone(),
      center: Vec3::ZERO,
      acceleration: point,
      magnitude: 4.0,
      epsilon: 0.0,
      max_radius: 3.0,
    };
    let mut pool: ParticlePool = ParticlePool::new(2, 1);

    pool.add(
      Vec3::new(2.0, 0.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    pool.add(
      Vec3::new(5.0, 0.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    action.execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(0.0, 1.0, 0.0));
    assert_eq!(pool.get_particles()[1].velocity, Vec3::ZERO);
  }
}
