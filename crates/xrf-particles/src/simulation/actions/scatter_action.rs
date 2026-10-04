use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_scatter::ParticleActionScatter;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAScatter`: accelerates particles straight away from a centre, falling off with the square distance.
pub(crate) struct ScatterAction {
  center_local: Vec3,
  center: Vec3,
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl ScatterAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.center = matrix.transform_point3(self.center_local);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;

    for m in pool.get_particles_mut() {
      let away: Vec3 = m.position - self.center;
      let distance_sqr: f32 = away.length_squared();

      if !is_limited || distance_sqr < max_radius_sqr {
        let direction: Vec3 = away / distance_sqr.sqrt();

        m.velocity += direction * (magdt / (distance_sqr + self.epsilon));
      }
    }
  }
}

impl From<&ParticleActionScatter> for ScatterAction {
  fn from(action: &ParticleActionScatter) -> Self {
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

  use super::ScatterAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn pushes_away_by_the_inverse_square_distance() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::new(0.0, 0.0, 2.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    ScatterAction {
      center_local: Vec3::ZERO,
      center: Vec3::ZERO,
      magnitude: 8.0,
      epsilon: 0.0,
      max_radius: 1.0e17,
    }
    .execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(0.0, 0.0, 2.0));
  }
}
