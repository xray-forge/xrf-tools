use glam::Mat4;

use crate::data::actions::particle_action_sink::ParticleActionSink;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PASink`: removes particles inside a volume, or outside it.
pub(crate) struct SinkAction {
  is_killing_inside: bool,
  position_local: ParticleVolume,
  position: ParticleVolume,
}

impl SinkAction {
  pub fn transform(&mut self, matrix: &Mat4) {
    self.position.transform_from(&self.position_local, matrix);
  }

  pub fn execute(&self, pool: &mut ParticlePool) {
    for index in (0..pool.len()).rev() {
      let position = pool.get_particles()[index].position;

      if self.position.is_within(position, pool.get_random()) == self.is_killing_inside {
        pool.remove(index);
      }
    }
  }
}

impl From<&ParticleActionSink> for SinkAction {
  fn from(action: &ParticleActionSink) -> Self {
    let position: ParticleVolume = ParticleVolume::from(&action.position);

    Self {
      is_killing_inside: action.kill_inside != 0,
      position_local: position.clone(),
      position,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::SinkAction;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  #[test]
  fn removes_particles_outside_a_kept_volume() {
    let ground: ParticleVolume = ParticleVolume {
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
    let mut pool: ParticlePool = ParticlePool::new(2, 1);

    pool.add(Vec3::Y, Vec3::ZERO, Vec3::ONE, Vec3::ZERO, Vec3::ZERO, Vec4::ONE, 0.0);
    pool.add(-Vec3::Y, Vec3::ZERO, Vec3::ONE, Vec3::ZERO, Vec3::ZERO, Vec4::ONE, 0.0);
    SinkAction {
      is_killing_inside: false,
      position_local: ground.clone(),
      position: ground,
    }
    .execute(&mut pool);

    assert_eq!(pool.len(), 1);
    assert_eq!(pool.get_particles()[0].position, Vec3::Y);
  }
}
