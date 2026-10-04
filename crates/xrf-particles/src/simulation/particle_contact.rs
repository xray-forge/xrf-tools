use glam::Vec3;

/// Where a particle's path meets a surface: how far along it, and the surface's unit normal.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleContact {
  pub distance: f32,
  pub normal: Vec3,
}
