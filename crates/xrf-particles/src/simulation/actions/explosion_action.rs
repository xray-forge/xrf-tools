use glam::{Mat4, Vec3};
use xrf_math::EPS;

use crate::data::actions::particle_action_explosion::ParticleActionExplosion;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_pool::ParticlePool;

/// `PAExplosion`: pushes particles out from a centre by a growing Gaussian shock wave.
pub(crate) struct ExplosionAction {
  center_local: Vec3,
  center: Vec3,
  velocity: f32,
  magnitude: f32,
  st_dev: f32,
  age: f32,
  epsilon: f32,
}

impl ExplosionAction {
  /// `ONEOVERSQRT2PI`.
  const ONE_OVER_SQRT_TWO_PI: f32 = 1.0 / 2.506_628_3;

  pub fn transform(&mut self, matrix: &Mat4) {
    self.center = matrix.transform_point3(self.center_local);
  }

  /// `PlayEffect` starts the wave over.
  pub fn play(&mut self) {
    self.age = 0.0;
  }

  pub fn execute(&mut self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let radius: f32 = self.velocity * self.age;
    let magdt: f32 = self.magnitude * step.dt;
    let one_over_sigma: f32 = 1.0 / self.st_dev;
    let inner_exponent: f32 = -0.5 * one_over_sigma * one_over_sigma;
    let outer_exponent: f32 = Self::ONE_OVER_SQRT_TWO_PI * one_over_sigma;

    for m in pool.get_particles_mut() {
      let direction: Vec3 = m.position - self.center;
      let dist_sqr: f32 = direction.length_squared();
      let dist: f32 = dist_sqr.sqrt();
      let from_wave_sqr: f32 = (radius - dist) * (radius - dist);
      let gd: f32 = (from_wave_sqr * inner_exponent).exp() * outer_exponent;

      m.velocity += direction * (gd * magdt / ((dist + EPS) * (dist_sqr + self.epsilon)));
    }

    self.age += step.dt;
  }
}

impl From<&ParticleActionExplosion> for ExplosionAction {
  fn from(action: &ParticleActionExplosion) -> Self {
    let center: Vec3 = Vec3::from_array(action.center.to_array());

    Self {
      center_local: center,
      center,
      velocity: action.velocity,
      magnitude: action.magnitude,
      st_dev: action.st_dev,
      age: action.age,
      epsilon: action.epsilon,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};
  use xrf_math::EPS;

  use super::ExplosionAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn pushes_a_particle_on_the_wave_front_and_ages_the_wave() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut action: ExplosionAction = ExplosionAction {
      center_local: Vec3::ZERO,
      center: Vec3::ZERO,
      velocity: 1.0,
      magnitude: 1.0,
      st_dev: 1.0,
      age: 2.0,
      epsilon: 0.0,
    };
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::new(2.0, 0.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    );
    action.execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    // On the front (radius 2) the Gaussian is its peak 1 / sqrt(2 pi).
    let expected: f32 = 2.0 * (ExplosionAction::ONE_OVER_SQRT_TWO_PI * 0.5 / ((2.0 + EPS) * 4.0));

    assert_eq!(pool.get_particles()[0].velocity.x, expected);
    assert_eq!(action.age, 2.5);
  }
}
