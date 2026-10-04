use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_vortex::ParticleActionVortex;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAVortex`: turns each particle's position about an axis through a centre, faster nearer it.
pub(crate) struct VortexAction {
  center_local: Vec3,
  axis_local: Vec3,
  center: Vec3,
  axis: Vec3,
  magnitude: f32,
  epsilon: f32,
  max_radius: f32,
}

impl VortexAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.center = matrix.transform_point3(self.center_local);
    self.axis = matrix.transform_vector3(self.axis_local);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let max_radius_sqr: f32 = self.max_radius * self.max_radius;
    let is_limited: bool = max_radius_sqr < ParticleActionStep::MAX_FLOAT;

    for m in pool.get_particles_mut() {
      let offset: Vec3 = m.position - self.center;
      let radius_sqr: f32 = offset.length_squared();

      if is_limited && radius_sqr > max_radius_sqr {
        continue;
      }

      let radius: f32 = radius_sqr.sqrt();
      let direction: Vec3 = offset / radius;
      let along: Vec3 = self.axis * direction.dot(self.axis);
      let across: Vec3 = direction - along;
      let side: Vec3 = self.axis.cross(across);
      let (sin, cos) = (magdt / (radius_sqr + self.epsilon)).sin_cos();

      m.position = (across * cos + side * sin + along) * radius + self.center;
    }
  }
}

impl From<&ParticleActionVortex> for VortexAction {
  fn from(action: &ParticleActionVortex) -> Self {
    let center: Vec3 = Vec3::from_array(action.center.to_array());
    let axis: Vec3 = Vec3::from_array(action.axis.to_array());

    Self {
      center_local: center,
      axis_local: axis,
      center,
      axis,
      magnitude: action.magnitude,
      epsilon: action.epsilon,
      max_radius: action.max_radius,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::f32::consts::FRAC_PI_2;

  use glam::{Vec3, Vec4};

  use super::VortexAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn turns_a_particle_about_the_axis_by_magnitude_over_square_radius() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::new(2.0, 1.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    // A quarter turn about y at radius sqrt(5): theta = 5 pi / 2 / 5.
    VortexAction {
      center_local: Vec3::ZERO,
      axis_local: Vec3::Y,
      center: Vec3::ZERO,
      axis: Vec3::Y,
      magnitude: 5.0 * FRAC_PI_2,
      epsilon: 0.0,
      max_radius: 1.0e17,
    }
    .execute(&mut pool, &ParticleActionStep::new(1.0, &rules));

    let position: Vec3 = pool.get_particles()[0].position;

    // y cross x is -z, so x turns to -z; the height along the axis stays.
    assert!((position - Vec3::new(0.0, 1.0, -2.0)).length() < 1e-5);
  }
}
