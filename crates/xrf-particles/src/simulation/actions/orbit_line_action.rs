use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_orbit_line::ParticleActionOrbitLine;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAOrbitLine`: accelerates particles towards the nearest point of a line.
pub(crate) struct OrbitLineAction {
  position_local: Vec3,
  axis_local: Vec3,
  position: Vec3,
  axis: Vec3,
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl OrbitLineAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.position = matrix.transform_point3(self.position_local);
    self.axis = matrix.transform_vector3(self.axis_local);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;

    for m in pool.get_particles_mut() {
      let from_base: Vec3 = m.position - self.position;
      let into: Vec3 = self.axis * from_base.dot(self.axis) - from_base;
      let distance_sqr: f32 = into.length_squared();

      // The engine divides by the distance plus its square, not their product.
      if !is_limited || distance_sqr < max_radius_sqr {
        m.velocity += into * (magdt / (distance_sqr.sqrt() + (distance_sqr + self.epsilon)));
      }
    }
  }
}

impl From<&ParticleActionOrbitLine> for OrbitLineAction {
  fn from(action: &ParticleActionOrbitLine) -> Self {
    let position: Vec3 = Vec3::from_array(action.position.to_array());
    let axis: Vec3 = Vec3::from_array(action.axis.to_array());

    Self {
      position_local: position,
      axis_local: axis,
      position,
      axis,
      magnitude: action.magnitude,
      epsilon: action.epsilon,
      max_radius: action.max_radius,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::OrbitLineAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn pulls_towards_the_line_by_distance_plus_its_square() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::new(2.0, 5.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    OrbitLineAction {
      position_local: Vec3::ZERO,
      axis_local: Vec3::Y,
      position: Vec3::ZERO,
      axis: Vec3::Y,
      magnitude: 6.0,
      epsilon: 0.0,
      max_radius: 1.0e17,
    }
    .execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    // into = (-2, 0, 0), 6 / (2 + 4) = 1.
    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(-2.0, 0.0, 0.0));
  }
}
