use glam::{Vec3, Vec4};
use xrf_engine_target::XrayEngine;

use crate::data::actions::particle_action_target_color::ParticleActionTargetColor;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle::Particle;
use crate::simulation::particle_pool::ParticlePool;

/// `PATargetColor`: shifts the colour of particles inside an age window, a share of `kill_old_time`, towards a target.
pub(crate) struct TargetColorAction {
  color: Vec4,
  scale: f32,
  time_from: f32,
  time_to: f32,
}

impl TargetColorAction {
  /// Monolith's `STEP_DEFAULT`: the step its colour shift is measured against.
  const STEP_DEFAULT: f32 = 0.033;

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let from: f32 = self.time_from * step.kill_old_time;
    let to: f32 = self.time_to * step.kill_old_time;

    match step.rules.get_engine() {
      // OpenXRay moves the packed colour a share of the step towards the target and packs it again.
      XrayEngine::Vanilla => {
        let share: f32 = self.scale * step.dt;

        for m in pool.get_particles_mut() {
          if m.age < from || m.age > to {
            continue;
          }

          m.color = Particle::quantize_color(m.color + (self.color - m.color) * share);
        }
      }
      // Monolith lerps by a share of its default step, packs that, and moves its float colour towards it by the step's
      // part of the default, which lands on it at the default step.
      XrayEngine::Extended => {
        let coefficient: f32 = Self::STEP_DEFAULT / step.dt;
        let share: f32 = self.scale * Self::STEP_DEFAULT;

        for m in pool.get_particles_mut() {
          if m.age < from || m.age > to {
            continue;
          }

          let next: Vec4 = Particle::quantize_color(m.color * (1.0 - share) + self.color * share);

          m.color -= (m.color - next) / coefficient;
        }
      }
    }
  }
}

impl From<&ParticleActionTargetColor> for TargetColorAction {
  fn from(action: &ParticleActionTargetColor) -> Self {
    let color: Vec3 = Vec3::from_array(action.color.to_array());

    Self {
      color: color.extend(action.alpha),
      scale: action.scale,
      time_from: action.time_from,
      time_to: action.time_to,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};
  use xrf_engine_target::XrayEngine;

  use super::TargetColorAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;

  fn shifted(engine: XrayEngine, dt: f32) -> Vec4 {
    let rules: ParticleEngineRules = ParticleEngineRules::new(engine, 1.0);
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ZERO,
      0.5,
    );
    TargetColorAction {
      color: Vec4::ONE,
      scale: 10.0,
      time_from: 0.0,
      time_to: 1.0,
    }
    .execute(&mut pool, &ParticleActionStep::new(dt, &rules));

    pool.get_particles()[0].color
  }

  #[test]
  fn moves_openxray_colours_a_share_of_the_step_and_packs_them() {
    // 10 * 0.033 = 0.33 of the way to white, floored to 84 of 255.
    assert_eq!(shifted(XrayEngine::Vanilla, 0.033), Vec4::splat(84.0 / 255.0));
  }

  #[test]
  fn moves_monolith_colours_by_the_steps_part_of_the_default() {
    // At half the default step Monolith moves half of the way to the packed 84 of 255.
    assert_eq!(shifted(XrayEngine::Extended, 0.033), Vec4::splat(84.0 / 255.0));
    assert_eq!(
      shifted(XrayEngine::Extended, 0.0165),
      Vec4::splat(84.0 / 255.0 / (0.033 / 0.0165))
    );
  }

  #[test]
  fn leaves_particles_outside_the_age_window() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);
    let mut step: ParticleActionStep = ParticleActionStep::new(0.033, &rules);

    step.kill_old_time = 0.25;
    pool.add(
      Vec3::ZERO,
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ZERO,
      0.5,
    );
    TargetColorAction {
      color: Vec4::ONE,
      scale: 10.0,
      time_from: 0.0,
      time_to: 1.0,
    }
    .execute(&mut pool, &step);

    assert_eq!(pool.get_particles()[0].color, Vec4::ZERO);
  }
}
