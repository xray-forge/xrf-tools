use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_bounce::ParticleActionBounce;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::engine_vector::EngineVector;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PABounce`: reflects particles crossing a surface this step, with friction and resilience.
pub(crate) struct BounceAction {
  position_local: ParticleVolume,
  position: ParticleVolume,
  one_minus_friction: f32,
  resilience: f32,
  cutoff_sqr: f32,
}

impl BounceAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.position.transform_from(&self.position_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let domain: &ParticleVolume = &self.position;
    let (particles, random) = pool.split_mut();

    match domain.kind {
      ParticleVolume::TRIANGLE | ParticleVolume::RECTANGLE => {
        let (s1, s2) = domain.get_plane_inverse_basis();

        for m in particles {
          let next: Vec3 = m.position + m.velocity * step.dt;
          let dist_old: f32 = m.position.dot(domain.p2) + domain.radius1;
          let dist_new: f32 = next.dot(domain.p2) + domain.radius1;

          if dist_old * dist_new >= 0.0 {
            continue;
          }

          let nv: f32 = domain.p2.dot(m.velocity);
          let t: f32 = -dist_old / nv;
          let offset: Vec3 = m.position + m.velocity * t - domain.p1;
          let upos: f32 = offset.dot(s1);
          let vpos: f32 = offset.dot(s2);

          let is_outside: bool = if domain.kind == ParticleVolume::TRIANGLE {
            upos < 0.0 || vpos < 0.0 || (upos + vpos) > 1.0
          } else {
            upos < 0.0 || upos > 1.0 || vpos < 0.0 || vpos > 1.0
          };

          if is_outside {
            continue;
          }

          let normal_speed: f32 = if domain.kind == ParticleVolume::TRIANGLE {
            nv
          } else {
            m.velocity.dot(domain.p2)
          };

          m.velocity = self.reflect(m.velocity, domain.p2 * normal_speed);
        }
      }
      ParticleVolume::DISC => {
        let r1_sqr: f32 = domain.radius1 * domain.radius1;
        let r2_sqr: f32 = domain.radius2 * domain.radius2;

        for m in particles {
          let next: Vec3 = m.position + m.velocity * step.dt;
          // A disc keeps its plane's `d` in `radius1Sqr`.
          let dist_old: f32 = m.position.dot(domain.p2) + domain.radius1_sqr;
          let dist_new: f32 = next.dot(domain.p2) + domain.radius1_sqr;

          if dist_old * dist_new >= 0.0 {
            continue;
          }

          let nv: f32 = domain.p2.dot(m.velocity);
          let t: f32 = -dist_old / nv;
          let radius_sqr: f32 = (m.position + m.velocity * t - domain.p1).length_squared();

          if radius_sqr > r1_sqr || radius_sqr < r2_sqr {
            continue;
          }

          m.velocity = self.reflect(m.velocity, domain.p2 * nv);
        }
      }
      ParticleVolume::PLANE => {
        for m in particles {
          let next: Vec3 = m.position + m.velocity * step.dt;
          let dist_old: f32 = m.position.dot(domain.p2) + domain.radius1;
          let dist_new: f32 = next.dot(domain.p2) + domain.radius1;

          if dist_old * dist_new >= 0.0 {
            continue;
          }

          m.velocity = self.reflect(m.velocity, domain.p2 * m.velocity.dot(domain.p2));
        }
      }
      ParticleVolume::SPHERE => {
        for m in particles {
          let next: Vec3 = m.position + m.velocity * step.dt;

          if !domain.is_within(next, random) {
            continue;
          }

          let was_inside: bool = domain.is_within(m.position, random);
          // The engine takes the normal at the particle, not where it meets the surface.
          let normal: Vec3 = (m.position - domain.p1).normalize_safe();
          let normal_speed: f32 = m.velocity.dot(normal);
          let normal_velocity: Vec3 = normal * normal_speed;

          if was_inside {
            if normal_speed < 0.0 {
              m.velocity = (m.velocity - normal_velocity) - normal_velocity;
            }
          } else {
            m.velocity = self.reflect(m.velocity, normal_velocity);
          }
        }
      }
      _ => {}
    }
  }

  /// The velocity heading out: normal part reversed by resilience, tangent slowed by friction above the cutoff.
  fn reflect(&self, velocity: Vec3, normal_velocity: Vec3) -> Vec3 {
    let tangent: Vec3 = velocity - normal_velocity;

    if tangent.length_squared() <= self.cutoff_sqr {
      tangent - normal_velocity * self.resilience
    } else {
      tangent * self.one_minus_friction - normal_velocity * self.resilience
    }
  }
}

impl From<&ParticleActionBounce> for BounceAction {
  fn from(action: &ParticleActionBounce) -> Self {
    let position: ParticleVolume = ParticleVolume::from(&action.position);

    Self {
      position_local: position.clone(),
      position,
      one_minus_friction: action.one_minus_friction,
      resilience: action.resilience,
      cutoff_sqr: action.cutoff_sqr,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::BounceAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn reflects_a_particle_crossing_a_plane_with_friction_and_resilience() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let floor: ParticleVolume = ParticleVolume {
      kind: ParticleVolume::PLANE,
      p1: Vec3::ZERO,
      p2: Vec3::Y,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    };
    let action: BounceAction = BounceAction {
      position_local: floor.clone(),
      position: floor,
      one_minus_friction: 0.5,
      resilience: 0.25,
      cutoff_sqr: 0.0,
    };
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(
      Vec3::new(0.0, 0.1, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::new(4.0, -8.0, 0.0),
      Vec4::ONE,
      0.0,
    );
    action.execute(&mut pool, &ParticleActionStep::new(0.033, &rules));

    // Tangent (4, 0, 0) halved, normal (0, -8, 0) reversed at a quarter.
    assert_eq!(pool.get_particles()[0].velocity, Vec3::new(2.0, 2.0, 0.0));
  }
}
