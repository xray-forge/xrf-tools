use glam::{Mat4, Vec3};

use crate::data::actions::particle_action_avoid::ParticleActionAvoid;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::engine_vector::EngineVector;
use crate::simulation::particle::Particle;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PAAvoid`: steers particles heading into a plane, rectangle, triangle, disc or sphere away from it, keeping speed.
pub(crate) struct AvoidAction {
  position_local: ParticleVolume,
  position: ParticleVolume,
  look_ahead: f32,
  magnitude: f32,
  epsilon: f32,
}

impl AvoidAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.position.transform_from(&self.position_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    let magdt: f32 = self.magnitude * step.dt;
    let domain: &ParticleVolume = &self.position;
    let particles: &mut [Particle] = pool.get_particles_mut();

    match domain.kind {
      ParticleVolume::PLANE => {
        let is_limited: bool = self.look_ahead < ParticleActionStep::MAX_FLOAT;

        for m in particles {
          let dist: f32 = m.position.dot(domain.p2) + domain.radius1;

          if is_limited && dist >= self.look_ahead {
            continue;
          }

          m.velocity = self.steer(m.velocity, domain.p2, magdt, dist);
        }
      }
      ParticleVolume::RECTANGLE | ParticleVolume::TRIANGLE => {
        let (u, v) = (domain.u, domain.v);
        let un: Vec3 = u / domain.radius1_sqr;
        let vn: Vec3 = v / domain.radius2_sqr;
        let fn_: Vec3 = (v - u).normalize_safe();
        let (s1, s2) = domain.get_plane_inverse_basis();

        for m in particles {
          let next: Vec3 = m.position + m.velocity * step.dt * self.look_ahead;
          let dist_old: f32 = m.position.dot(domain.p2) + domain.radius1;
          let dist_new: f32 = next.dot(domain.p2) + domain.radius1;

          if dist_old * dist_new >= 0.0 {
            continue;
          }

          let t: f32 = -dist_old / domain.p2.dot(m.velocity);
          let offset: Vec3 = m.position + m.velocity * t - domain.p1;
          let upos: f32 = offset.dot(s1);
          let vpos: f32 = offset.dot(s2);

          let safety: Vec3 = if domain.kind == ParticleVolume::RECTANGLE {
            if upos < 0.0 || vpos < 0.0 || upos > 1.0 || vpos > 1.0 {
              continue;
            }

            let uofs: Vec3 = un * un.dot(offset) - offset;
            let vofs: Vec3 = vn * vn.dot(offset) - offset;
            let far_offset: Vec3 = (u + v) - offset;
            let fofs: Vec3 = un * un.dot(far_offset) - far_offset;
            // The engine projects on `un` again where the fourth edge wants `vn`.
            let gofs: Vec3 = un * un.dot(far_offset) - far_offset;
            let (udist, vdist, fdist, gdist) = (
              uofs.length_squared(),
              vofs.length_squared(),
              fofs.length_squared(),
              gofs.length_squared(),
            );

            if udist <= vdist && udist <= fdist && udist <= gdist {
              uofs
            } else if vdist <= fdist && vdist <= gdist {
              vofs
            } else if fdist <= gdist {
              fofs
            } else {
              gofs
            }
          } else {
            if upos < 0.0 || vpos < 0.0 || (upos + vpos) > 1.0 {
              continue;
            }

            let uofs: Vec3 = un * un.dot(offset) - offset;
            let vofs: Vec3 = vn * vn.dot(offset) - offset;
            let far_offset: Vec3 = offset - u;
            let fofs: Vec3 = fn_ * fn_.dot(far_offset) - far_offset;
            let (udist, vdist, fdist) = (uofs.length_squared(), vofs.length_squared(), fofs.length_squared());

            if udist <= vdist && udist <= fdist {
              uofs
            } else if vdist <= fdist {
              vofs
            } else {
              fofs
            }
          };

          m.velocity = self.steer(m.velocity, safety.normalize_safe(), magdt, t);
        }
      }
      ParticleVolume::DISC => {
        let r1_sqr: f32 = domain.radius1 * domain.radius1;
        let r2_sqr: f32 = domain.radius2 * domain.radius2;

        for m in particles {
          let next: Vec3 = m.position + m.velocity * step.dt * self.look_ahead;
          // A disc keeps its plane's `d` in `radius1Sqr`.
          let dist_old: f32 = m.position.dot(domain.p2) + domain.radius1_sqr;
          let dist_new: f32 = next.dot(domain.p2) + domain.radius1_sqr;

          if dist_old * dist_new >= 0.0 {
            continue;
          }

          let t: f32 = -dist_old / domain.p2.dot(m.velocity);
          let offset: Vec3 = m.position + m.velocity * t - domain.p1;
          let radius_sqr: f32 = offset.length_squared();

          if radius_sqr > r1_sqr || radius_sqr < r2_sqr {
            continue;
          }

          m.velocity = self.steer(m.velocity, offset.normalize_safe(), magdt, t);
        }
      }
      ParticleVolume::SPHERE => {
        let r_sqr: f32 = domain.radius1 * domain.radius1;

        for m in particles {
          let vm: f32 = m.velocity.length();
          let vn: Vec3 = m.velocity / vm;
          let to_center: Vec3 = domain.p1 - m.position;
          let along: f32 = to_center.dot(vn);
          let disc: f32 = r_sqr - to_center.dot(to_center) + along * along;

          if disc < 0.0 {
            continue;
          }

          let t: f32 = along - disc.sqrt();

          if t < 0.0 || t > vm * self.look_ahead {
            continue;
          }

          let side: Vec3 = vn.cross(to_center).normalize_safe();
          let safety: Vec3 = vn.cross(side);
          let blended: Vec3 = safety * (magdt / (t * t + self.epsilon)) + vn;

          m.velocity = blended * (vm / blended.length());
        }
      }
      _ => {}
    }
  }

  /// Blends a direction to safety into the velocity by how near the surface is, keeping its speed.
  fn steer(&self, velocity: Vec3, safety: Vec3, magdt: f32, near: f32) -> Vec3 {
    let vm: f32 = velocity.length();
    let vn: Vec3 = velocity / vm;
    let blended: Vec3 = safety * (magdt / (near * near + self.epsilon)) + vn;

    blended * (vm / blended.length())
  }
}

impl From<&ParticleActionAvoid> for AvoidAction {
  fn from(action: &ParticleActionAvoid) -> Self {
    let position: ParticleVolume = ParticleVolume::from(&action.position);

    Self {
      position_local: position.clone(),
      position,
      look_ahead: action.look_ahead,
      magnitude: action.magnitude,
      epsilon: action.epsilon,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::AvoidAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn bends_a_particle_near_a_plane_keeping_its_speed() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(1, 1);
    let action: AvoidAction = AvoidAction {
      position_local: plane(),
      position: plane(),
      look_ahead: 10.0,
      magnitude: 1.0,
      epsilon: 0.0,
    };

    pool.add(
      Vec3::new(0.0, 1.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::new(2.0, 0.0, 0.0),
      Vec4::ONE,
      0.0,
    );
    action.execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    // dist = 1, so the up normal blends in by 0.5 / 1 against the unit direction; the speed of 2 is kept.
    let expected: Vec3 = Vec3::new(1.0, 0.5, 0.0) * (2.0 / Vec3::new(1.0, 0.5, 0.0).length());

    assert_eq!(pool.get_particles()[0].velocity, expected);
  }

  fn plane() -> ParticleVolume {
    ParticleVolume {
      kind: ParticleVolume::PLANE,
      p1: Vec3::ZERO,
      p2: Vec3::Y,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    }
  }
}
