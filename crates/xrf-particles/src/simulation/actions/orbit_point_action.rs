use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_orbit_point::ParticleActionOrbitPoint;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAOrbitPoint`: accelerates particles towards a point.
pub(crate) struct OrbitPointAction {
  center_local: Vec3,
  center: Vec3,
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl OrbitPointAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.center = matrix.transform_point3(self.center_local);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;

    for m in pool.get_particles_mut() {
      let to_center: Vec3 = self.center - m.position;
      let distance_sqr: f32 = to_center.length_squared();

      // The engine divides by the distance plus its square, not their product.
      if !is_limited || distance_sqr < max_radius_sqr {
        m.velocity += to_center * (magdt / (distance_sqr.sqrt() + (distance_sqr + self.epsilon)));
      }
    }
  }
}

impl From<&ParticleActionOrbitPoint> for OrbitPointAction {
  fn from(action: &ParticleActionOrbitPoint) -> Self {
    let center: Vec3 = Vec3::from_array(action.center.to_array());

    Self {
      center_local: center,
      center,
      magnitude: action.magnitude,
      epsilon: action.epsilon,
      max_radius: action.max_radius,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::OrbitPointAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn pulls_only_particles_inside_the_radius() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
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
      Vec3::new(9.0, 0.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    OrbitPointAction {
      center_local: Vec3::ZERO,
      center: Vec3::ZERO,
      magnitude: 6.0,
      epsilon: 0.0,
      max_radius: 3.0,
    }
    .execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(-2.0, 0.0, 0.0));
    assert_eq!(pool.get_particles()[1].velocity, Vec3::ZERO);
  }
}
