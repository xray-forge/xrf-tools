use xrf_math::glam::Vec3;

/// A point's rays' origin, the skip code of the face they pass, and the point itself (`Origin`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RayGpuOrigin {
  lifted: [f32; 3],
  skip: u32,
  point: [f32; 3],
  _pad: u32,
}

impl RayGpuOrigin {
  /// A point's origin: its rays starting at `lifted`, aimed from `point`, passing the face of skip code `skip`.
  pub(crate) fn new(lifted: Vec3, point: Vec3, skip: u32) -> Self {
    Self {
      lifted: lifted.to_array(),
      skip,
      point: point.to_array(),
      _pad: 0,
    }
  }
}
