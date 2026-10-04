use crate::data::actions::particle_action_copy_vertex::ParticleActionCopyVertex;
use crate::simulation::particle_pool::ParticlePool;

/// `PACopyVertexB`: sets every particle's previous position to its position.
pub(crate) struct CopyVertexAction {
  is_copying_position: bool,
}

impl CopyVertexAction {
  pub fn execute(&self, pool: &mut ParticlePool) {
    if self.is_copying_position {
      for m in pool.get_particles_mut() {
        m.previous_position = m.position;
      }
    }
  }
}

impl From<&ParticleActionCopyVertex> for CopyVertexAction {
  fn from(action: &ParticleActionCopyVertex) -> Self {
    Self {
      is_copying_position: action.copy_position != 0,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::CopyVertexAction;
  use crate::simulation::particle_pool::ParticlePool;

  #[test]
  fn copies_the_position_into_the_previous_one() {
    let mut pool: ParticlePool = ParticlePool::new(1, 1);

    pool.add(Vec3::X, Vec3::ZERO, Vec3::ONE, Vec3::ZERO, Vec3::ZERO, Vec4::ONE, 0.0);
    CopyVertexAction {
      is_copying_position: true,
    }
    .execute(&mut pool);

    assert_eq!(pool.get_particles()[0].previous_position, Vec3::X);
  }
}
