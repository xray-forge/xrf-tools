use xrf_math::glam;

/// The workspace's vectors (glam's) as the engine's and back: the one place the two meet.
pub(crate) struct JoltVectors;

impl JoltVectors {
  pub(crate) fn create_vector(value: glam::Vec3) -> oxijolt::Vec3 {
    oxijolt::Vec3::new(value.x, value.y, value.z)
  }

  pub(crate) fn create_position(value: glam::Vec3) -> oxijolt::RVec3 {
    oxijolt::RVec3::new(value.x, value.y, value.z)
  }

  pub(crate) fn read_position(value: oxijolt::RVec3) -> glam::Vec3 {
    glam::Vec3::new(value.x, value.y, value.z)
  }
}
