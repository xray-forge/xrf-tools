use glam::Vec3;

/// A point a lightmap texel is lit at: the texel, the mesh face it lies on, which its shadow rays pass, where it lies
/// and what it faces, and whether it is a walk along the face's edge rather than one of the texel's samples.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LightTexelPoint {
  pub(crate) texel: u32,
  pub(crate) face: u32,
  pub(crate) point: Vec3,
  pub(crate) normal: Vec3,
  pub(crate) is_edge: bool,
}
