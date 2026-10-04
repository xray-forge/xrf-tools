use glam::Vec3;
use xrf_engine_target::XrayEngine;

use crate::data::actions::particle_action_turbulence::ParticleActionTurbulence;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle_noise::ParticleNoise;
use crate::simulation::particle_pool::ParticlePool;

/// `PATurbulence`: turns each particle's velocity by a drifting noise gradient, keeping its speed.
pub(crate) struct TurbulenceAction {
  frequency: f32,
  octaves: i32,
  magnitude: f32,
  epsilon: f32,
  offset: Vec3,
  age: f32,
}

impl TurbulenceAction {
  /// `PlayEffect` starts the drift over.
  pub fn play(&mut self) {
    self.age = 0.0;
  }

  pub fn execute(&mut self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    self.age += step.dt;

    if pool.is_empty() {
      return;
    }

    let noise: &ParticleNoise = ParticleNoise::get();
    // Monolith scales the push by its update coefficient.
    let magnitude: f32 = match step.rules.get_engine() {
      XrayEngine::Vanilla => self.magnitude,
      XrayEngine::Extended => self.magnitude * step.rules.get_update_coefficient(),
    };

    for m in pool.get_particles_mut() {
      let point: Vec3 = m.position + self.offset * self.age;
      let here: f32 = noise.fractal_sum(point, self.frequency, self.octaves);
      let gradient: Vec3 = Vec3::new(
        noise.fractal_sum(point + Vec3::X * self.epsilon, self.frequency, self.octaves),
        noise.fractal_sum(point + Vec3::Y * self.epsilon, self.frequency, self.octaves),
        noise.fractal_sum(point + Vec3::Z * self.epsilon, self.frequency, self.octaves),
      );
      let speed_before: f32 = m.velocity.length();

      m.velocity += (gradient - Vec3::splat(here)) * magnitude;
      m.velocity *= speed_before / m.velocity.length();
    }
  }
}

impl From<&ParticleActionTurbulence> for TurbulenceAction {
  fn from(action: &ParticleActionTurbulence) -> Self {
    Self {
      frequency: action.frequency,
      octaves: action.octaves,
      magnitude: action.magnitude,
      epsilon: action.epsilon,
      offset: Vec3::from_array(action.offset.to_array()),
      age: 0.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::TurbulenceAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_noise::ParticleNoise;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn turns_the_velocity_by_the_noise_gradient_keeping_its_speed() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);
    let mut action: TurbulenceAction = TurbulenceAction {
      frequency: 1.0,
      octaves: 2,
      magnitude: 3.0,
      epsilon: 0.1,
      offset: Vec3::X,
      age: 0.0,
    };
    let start: Vec3 = Vec3::new(0.3, 0.7, 0.2);

    pool.add(
      start,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::new(0.0, 0.0, 2.0),
      Vec4::ONE,
      0.0,
    );
    action.execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    let noise: &ParticleNoise = ParticleNoise::get();
    let point: Vec3 = start + Vec3::X * 0.5;
    let here: f32 = noise.fractal_sum(point, 1.0, 2);
    let pushed: Vec3 = Vec3::new(0.0, 0.0, 2.0)
      + (Vec3::new(
        noise.fractal_sum(point + Vec3::X * 0.1, 1.0, 2),
        noise.fractal_sum(point + Vec3::Y * 0.1, 1.0, 2),
        noise.fractal_sum(point + Vec3::Z * 0.1, 1.0, 2),
      ) - Vec3::splat(here))
        * 3.0;
    let velocity: Vec3 = pool.get_particles()[0].velocity;

    assert_eq!(velocity, pushed * (2.0 / pushed.length()));
    assert!((velocity.length() - 2.0).abs() < 1e-5);
    assert_eq!(action.age, 0.5);
  }
}
